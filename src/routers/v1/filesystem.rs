use std::io::ErrorKind;
use std::path::PathBuf;
use std::str::FromStr;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use bytes::Bytes;
use serde::Deserialize;
use crate::{AppState, db};
use crate::filesystem::{Filesystem, FSError, UserScopedFS};
use crate::routers::extractors::SessionUser;
use super::schema::{DataResponse, FSDirListing, FSQuota, FSTree};
use super::utils::{B64ToStrError, from_b64};

pub fn get_router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/quota", get(get_fs_quota))
        .route("/dir/get", get(get_dir_listing))
        .route("/dir/create", get(create_dir))
        .route("/file/get", get(get_file))
        .route("/file/write", post(write_file))
        .route("/cp", get(copy_item))
        .route("/rm", get(remove_item))
        .route("/rename", get(move_item))
        .route("/tree", get(get_usfs_tree))
}


enum FSInteractionError {
    FS(FSError),
    B64Decoding(B64ToStrError),
}


impl IntoResponse for FSInteractionError {
    fn into_response(self) -> Response {
        match self {
            Self::FS(err @ FSError::NotEnoughStorage) => (StatusCode::PAYLOAD_TOO_LARGE, err.to_string()),
            Self::FS(err @ (FSError::InvalidUTF8Path | FSError::PathBreaksOut)) => (StatusCode::BAD_REQUEST, err.to_string()),
            Self::FS(FSError::HFS(hfs_err)) =>
                match hfs_err.kind() {
                    ErrorKind::NotFound => (StatusCode::NOT_FOUND, "item at such path does not exist".to_string()),
                    ErrorKind::AlreadyExists => (StatusCode::CONFLICT, "item at such path already exists".to_string()),
                    _ => (StatusCode::INTERNAL_SERVER_ERROR, format!("unhandled host fs error: {hfs_err}"))
                }
            Self::B64Decoding(dec_error) => (StatusCode::BAD_REQUEST, format!("path decoding error: {dec_error}")),
        }.into_response()
    }
}


async fn get_fs_quota(
    State(AppState { filesystem, .. }): State<AppState>,
    SessionUser(user): SessionUser
) -> Result<Json<DataResponse<FSQuota>>, FSInteractionError> {
    let usfs = mk_usfs(&filesystem, &user).await?;

    Ok(Json(DataResponse::new(FSQuota::new(&usfs, &user).await.map_err(FSInteractionError::FS)?)))
}


#[derive(Deserialize)]
struct Path {
    #[serde(rename = "path")]
    path_enc: String,
}


async fn get_dir_listing(
    State(AppState { filesystem, .. }): State<AppState>,
    SessionUser(user): SessionUser,
    Query(Path { path_enc }): Query<Path>,
) -> Result<Json<DataResponse<FSDirListing>>, FSInteractionError> {
    let path = dec_path(&path_enc)?;
    let usfs = mk_usfs(&filesystem, &user).await?;

    Ok(Json(DataResponse::new(FSDirListing::new(&usfs, &path).await.map_err(FSInteractionError::FS)?)))
}


async fn create_dir(
    State(AppState { filesystem, .. }): State<AppState>,
    SessionUser(user): SessionUser,
    Query(Path { path_enc }): Query<Path>,
) -> Result<Json<DataResponse<()>>, FSInteractionError> {
    let path = dec_path(&path_enc)?;
    let usfs = mk_usfs(&filesystem, &user).await?;

    usfs.create_dir(&path).await.map_err(FSInteractionError::FS)?;

    Ok(Json(DataResponse::new(())))
}


async fn get_file(
    State(AppState { filesystem, .. }): State<AppState>,
    SessionUser(user): SessionUser,
    Query(Path { path_enc }): Query<Path>,
) -> Result<Vec<u8>, FSInteractionError> {
    let path = dec_path(&path_enc)?;
    let usfs = mk_usfs(&filesystem, &user).await?;

    usfs.read_file(&path).await.map_err(FSInteractionError::FS)
}


async fn write_file(
    State(AppState { filesystem, .. }): State<AppState>,
    SessionUser(user): SessionUser,
    Query(Path { path_enc }): Query<Path>,
    data: Bytes,
) -> Result<(), FSInteractionError> {
    let path = dec_path(&path_enc)?;
    let usfs = mk_usfs(&filesystem, &user).await?;

    usfs.write_file(&path, &data).await.map_err(FSInteractionError::FS)?;

    Ok(())
}


#[derive(Deserialize)]
struct Target {
    #[serde(rename = "target")]
    target_enc: String,
}


async fn copy_item(
    State(AppState { filesystem, .. }): State<AppState>,
    SessionUser(user): SessionUser,
    Query(Path { path_enc: source_enc }): Query<Path>,
    Query(Target { target_enc }): Query<Target>,
) -> Result<(), FSInteractionError> {
    let source = dec_path(&source_enc)?;
    let target = dec_path(&target_enc)?;
    let usfs = mk_usfs(&filesystem, &user).await?;

    usfs.copy_item(&source, &target).await.map_err(FSInteractionError::FS)?;

    Ok(())
}


async fn remove_item(
    State(AppState { filesystem, .. }): State<AppState>,
    SessionUser(user): SessionUser,
    Query(Path { path_enc }): Query<Path>,
) -> Result<(), FSInteractionError> {
    let path = dec_path(&path_enc)?;
    let usfs = mk_usfs(&filesystem, &user).await?;

    usfs.remove_item(&path).await.map_err(FSInteractionError::FS)?;

    Ok(())
}

#[derive(Deserialize)]
struct ItemMove {
    #[serde(rename = "oldpath")]
    source_enc: String,
    #[serde(rename = "newpath")]
    target_enc: String,
}


async fn move_item(
    State(AppState { filesystem, .. }): State<AppState>,
    SessionUser(user): SessionUser,
    Query(ItemMove { source_enc, target_enc }): Query<ItemMove>,
) -> Result<(), FSInteractionError> {
    let source = dec_path(&source_enc)?;
    let target = dec_path(&target_enc)?;
    let usfs = mk_usfs(&filesystem, &user).await?;
    
    usfs.move_item(&source, &target).await.map_err(FSInteractionError::FS)?;
    
    Ok(())
}


async fn get_usfs_tree(
    State(AppState { filesystem, .. }): State<AppState>,
    SessionUser(user): SessionUser,
) -> Result<Json<DataResponse<FSTree>>, FSInteractionError> {
    let usfs = mk_usfs(&filesystem, &user).await?;
    
    Ok(Json(DataResponse::new(FSTree::new(&usfs, &PathBuf::from_str(".").unwrap()).await.map_err(FSInteractionError::FS)?)))
}


fn dec_path(s: &str) -> Result<PathBuf, FSInteractionError> {
    Ok(PathBuf::from_str(&from_b64(s).map_err(FSInteractionError::B64Decoding)?).unwrap())
}


async fn mk_usfs<'a>(fs: &'a Filesystem, user: &db::User) -> Result<UserScopedFS<'a>, FSInteractionError> {
    UserScopedFS::new(&fs, user.id).await.map_err(FSInteractionError::FS)
}
