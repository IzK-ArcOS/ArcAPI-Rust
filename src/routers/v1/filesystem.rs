use std::path::PathBuf;
use std::str::FromStr;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use bytes::Bytes;
use serde::Deserialize;
use crate::AppState;
use crate::filesystem::{FSError, UserScopedFS};
use crate::routers::extractors::SessionUser;
use crate::routers::v1::schema::{DataResponse, FSDirListing, FSQuota, FSTree};
use crate::routers::v1::utils::{B64ToStrError, from_b64};

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
            // fixme actually custom-handle some of the fs errors (specify )
            Self::FS(fs_error) => (StatusCode::BAD_REQUEST, format!("fs error: {fs_error}")).into_response(),
            Self::B64Decoding(dec_error) => (StatusCode::BAD_REQUEST, format!("path decoding error: {dec_error}")).into_response(),
        }
    }
}


async fn get_fs_quota(
    State(AppState { filesystem, .. }): State<AppState>,
    SessionUser(user): SessionUser
) -> Result<Json<DataResponse<FSQuota>>, FSInteractionError> {
    let usfs = UserScopedFS::new(&filesystem, user.id).await.map_err(FSInteractionError::FS)?;

    Ok(Json(DataResponse::new(FSQuota::new(&usfs, &user).await.map_err(FSInteractionError::FS)?)))
}


#[derive(Deserialize)]
struct Path {
    path: String,
}


async fn get_dir_listing(
    State(AppState { filesystem, .. }): State<AppState>,
    SessionUser(user): SessionUser,
    Query(Path { path: path_enc }): Query<Path>,
) -> Result<Json<DataResponse<FSDirListing>>, FSInteractionError> {
    // todo do smth about these 2 repetitive lines
    let path = PathBuf::from_str(&from_b64(&path_enc).map_err(FSInteractionError::B64Decoding)?).unwrap();
    let usfs = UserScopedFS::new(&filesystem, user.id).await.map_err(FSInteractionError::FS)?;

    Ok(Json(DataResponse::new(FSDirListing::new(&usfs, &path).await.map_err(FSInteractionError::FS)?)))
}


async fn create_dir(
    State(AppState { filesystem, .. }): State<AppState>,
    SessionUser(user): SessionUser,
    Query(Path { path: path_enc }): Query<Path>,
) -> Result<Json<DataResponse<()>>, FSInteractionError> {
    let path = PathBuf::from_str(&from_b64(&path_enc).map_err(FSInteractionError::B64Decoding)?).unwrap();
    let usfs = UserScopedFS::new(&filesystem, user.id).await.map_err(FSInteractionError::FS)?;

    usfs.create_dir(&path).await.map_err(FSInteractionError::FS)?;

    Ok(Json(DataResponse::new(())))
}


async fn get_file(
    State(AppState { filesystem, .. }): State<AppState>,
    SessionUser(user): SessionUser,
    Query(Path { path: path_enc }): Query<Path>,
) -> Result<Vec<u8>, FSInteractionError> {
    let path = PathBuf::from_str(&from_b64(&path_enc).map_err(FSInteractionError::B64Decoding)?).unwrap();
    let usfs = UserScopedFS::new(&filesystem, user.id).await.map_err(FSInteractionError::FS)?;

    Ok(usfs.read_file(&path).await.map_err(FSInteractionError::FS)?)
}


async fn write_file(
    State(AppState { filesystem, .. }): State<AppState>,
    SessionUser(user): SessionUser,
    Query(Path { path: path_enc }): Query<Path>,
    data: Bytes,
) -> Result<(), FSInteractionError> {
    let path = PathBuf::from_str(&from_b64(&path_enc).map_err(FSInteractionError::B64Decoding)?).unwrap();
    let usfs = UserScopedFS::new(&filesystem, user.id).await.map_err(FSInteractionError::FS)?;

    usfs.write_file(&path, &data).await.map_err(FSInteractionError::FS)?;

    Ok(())
}


#[derive(Deserialize)]
struct Target {
    target: String,
}


async fn copy_item(
    State(AppState { filesystem, .. }): State<AppState>,
    SessionUser(user): SessionUser,
    Query(Path { path: source_enc }): Query<Path>,
    Query(Target { target: target_enc }): Query<Target>,
) -> Result<(), FSInteractionError> {
    let source = PathBuf::from_str(&from_b64(&source_enc).map_err(FSInteractionError::B64Decoding)?).unwrap();
    let target = PathBuf::from_str(&from_b64(&target_enc).map_err(FSInteractionError::B64Decoding)?).unwrap();
    let usfs = UserScopedFS::new(&filesystem, user.id).await.map_err(FSInteractionError::FS)?;

    usfs.copy_item(&source, &target).await.map_err(FSInteractionError::FS)?;

    Ok(())
}


async fn remove_item(
    State(AppState { filesystem, .. }): State<AppState>,
    SessionUser(user): SessionUser,
    Query(Path { path: path_enc }): Query<Path>,
) -> Result<(), FSInteractionError> {
    let path = PathBuf::from_str(&from_b64(&path_enc).map_err(FSInteractionError::B64Decoding)?).unwrap();
    let usfs = UserScopedFS::new(&filesystem, user.id).await.map_err(FSInteractionError::FS)?;

    usfs.remove_item(&path).await.map_err(FSInteractionError::FS)?;

    Ok(())
}

#[derive(Deserialize)]
struct ItemRename {
    #[serde(rename = "oldpath")]
    old_path: String,
    #[serde(rename = "newpath")]
    new_path: String,
}


async fn move_item(
    State(AppState { filesystem, .. }): State<AppState>,
    SessionUser(user): SessionUser,
    Query(ItemRename { old_path: source_enc, new_path: target_enc }): Query<ItemRename>,
) -> Result<(), FSInteractionError> {
    let source = PathBuf::from_str(&from_b64(&source_enc).map_err(FSInteractionError::B64Decoding)?).unwrap();
    let target = PathBuf::from_str(&from_b64(&target_enc).map_err(FSInteractionError::B64Decoding)?).unwrap();
    let usfs = UserScopedFS::new(&filesystem, user.id).await.map_err(FSInteractionError::FS)?;
    
    usfs.move_item(&source, &target).await.map_err(FSInteractionError::FS)?;
    
    Ok(())
}


async fn get_usfs_tree(
    State(AppState { filesystem, .. }): State<AppState>,
    SessionUser(user): SessionUser,
) -> Result<Json<DataResponse<FSTree>>, FSInteractionError> {
    let usfs = UserScopedFS::new(&filesystem, user.id).await.map_err(FSInteractionError::FS)?;
    
    Ok(Json(DataResponse::new(FSTree::new(&usfs, &PathBuf::from_str(".").unwrap()).await.map_err(FSInteractionError::FS)?)))
}
