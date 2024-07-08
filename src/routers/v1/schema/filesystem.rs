// todo normalise paths in scoped_path (requires implementing respective method in fs)
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use crate::db;
use crate::filesystem::{FSRes, UserScopedFS};


const DEFAULT_MIME_TYPE: &str = "text/plain; charset=utf-8"; 


#[derive(Serialize, Deserialize)]
pub struct FSQuota {
    pub username: String,
    pub max: u64,
    pub used: u64,
    pub free: u64,
}


impl FSQuota {
    pub async fn new(usfs: &UserScopedFS<'_>, user: &db::User) -> FSRes<Self> {
        let max = usfs.fs().userspace_size().unwrap_or(u64::MAX);
        let used = usfs.get_item_size(&PathBuf::from_str(".").unwrap()).await?;

        Ok(Self {
            username: user.get_username(),
            free: max - used,
            max, used
        })
    }
}


#[derive(Serialize, Deserialize)]
pub struct FSFile {
    pub filename: String,
    #[serde(rename = "scopedPath")]
    pub scoped_path: String,
    pub size: u64,
    pub mime: String,
    #[serde(rename = "dateCreated")]
    pub date_created: i64,
    #[serde(rename = "dateModified")]
    pub date_modified: i64,
}


impl FSFile {
    pub async fn new(usfs: &UserScopedFS<'_>, path: PathBuf) -> FSRes<Self> {
        let (date_created, date_modified) = usfs.get_item_time_info(&path).await?;
        
        Ok(Self {
            filename: get_item_name(&path),
            scoped_path: path.to_string_lossy().to_string(),
            size: usfs.get_item_size(&path).await?,
            mime: usfs.get_mime(&path).await?.unwrap_or(DEFAULT_MIME_TYPE.to_string()),
            date_created: adapt_system_time_to_ms_timestamp(date_created),
            date_modified: adapt_system_time_to_ms_timestamp(date_modified),
        })
    }
}


#[derive(Serialize, Deserialize)]
pub struct FSDirectory {
    pub name: String,
    #[serde(rename = "scopedPath")]
    pub scoped_path: String,
}


impl FSDirectory {
    pub fn new(path: &Path) -> Self {
        Self {
            name: get_item_name(path),
            scoped_path: path.to_string_lossy().to_string(),
        }
    }
}


#[derive(Serialize, Deserialize)]
pub struct FSDirListing {
    pub name: String,
    #[serde(rename = "scopedPath")]
    pub scoped_path: String,
    pub files: Vec<FSFile>,
    pub directories: Vec<FSDirectory>,
}


impl FSDirListing {
    pub async fn new(usfs: &UserScopedFS<'_>, path: &Path) -> FSRes<Self> {
        let (files, directories) = usfs.list_dir(path).await?;
        
        Ok(Self {
            name: get_item_name(path),
            scoped_path: path.to_string_lossy().to_string(),
            files: futures::future::join_all(files.into_iter().map(|fp| FSFile::new(usfs, fp))).await.into_iter().collect::<Result<_, _>>()?,  // fixme remove the need to move the file path ...or not, idk
            directories: directories.into_iter().map(|dp| FSDirectory::new(&dp)).collect(),
        })
    }
}


#[derive(Serialize, Deserialize)]
pub struct FSPartialEntry {
    #[serde(rename = "scopedPath")]
    pub scoped_path: String,
    pub mime: String,
    pub filename: String,
}


impl FSPartialEntry {
    pub async fn new(usfs: &UserScopedFS<'_>, path: PathBuf) -> FSRes<Self> {
        Ok(Self {
            scoped_path: path.to_string_lossy().to_string(),
            mime: usfs.get_mime(&path).await?.unwrap_or(DEFAULT_MIME_TYPE.to_string()),
            filename: get_item_name(&path),
        })
    }
}


#[derive(Serialize, Deserialize)]
pub struct FSTree(pub Vec<FSPartialEntry>);


impl FSTree {
    pub async fn new(usfs: &UserScopedFS<'_>, path: &Path) -> FSRes<Self> {
        Ok(Self(futures::future::join_all(usfs.get_dir_tree(path).await?.into_iter().map(|p| FSPartialEntry::new(usfs, p))).await.into_iter().collect::<Result<_, _>>()?))
    }
} 


fn get_item_name(path: &Path) -> String {
    path.file_name().map(|r#fn| r#fn.to_string_lossy().to_string()).unwrap_or(".".to_string())
}


fn adapt_system_time_to_ms_timestamp(system_time: SystemTime) -> i64 {
    chrono::TimeDelta::from_std(system_time.duration_since(SystemTime::UNIX_EPOCH).unwrap()).unwrap().num_milliseconds()
}
