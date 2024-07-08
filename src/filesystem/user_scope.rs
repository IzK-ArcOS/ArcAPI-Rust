use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::SystemTime;
use normalize_path::NormalizePath;
use super::{Filesystem, FSError, FSRes};

pub struct UserScopedFS<'a> {
    fs: &'a Filesystem,
    user_id: i32,
    base_path: PathBuf,
}


impl<'a> UserScopedFS<'a> {
    pub async fn new(fs: &'a Filesystem, user_id: i32) -> FSRes<Self> {
        let self_ = Self {
            fs, user_id,
            base_path: PathBuf::from_str(&user_id.to_string()).unwrap(),
        };
        
        // yes i know this is some very weird code
        if !fs.construct_path(&self_.construct_path(".".as_ref())?)?.exists() {
            let td_res = self_.deploy_template().await;
            
            if let Some(res) = td_res {
                res?;
            } else {
                self_.create_dir(".".as_ref()).await?;
            };
        };
        
        Ok(self_)
    }

    pub fn fs(&self) -> &'a Filesystem {
        self.fs
    }

    pub fn user_id(&self) -> i32 {
        self.user_id
    }

    pub async fn create_dir(&self, path: &Path) -> FSRes<()> {
        self.fs.create_dir(&self.construct_path(path)?).await
    }

    pub async fn list_dir(&self, path: &Path) -> FSRes<(Vec<PathBuf>, Vec<PathBuf>)> {
        let (files, directories) =
            self.fs.list_dir(&self.construct_path(path)?).await?;

        Ok((
            self.adapt_paths(files).await?,
            self.adapt_paths(directories).await?,
        ))
    }

    pub async fn write_file(&self, path: &Path, data: &[u8]) -> FSRes<()> {
        if let Some(total_size) = self.fs.userspace_size() {
            if self.get_item_size(".".as_ref()).await? + data.len() as u64 > total_size {
                return Err(FSError::NotEnoughStorage);
            }
        };

        self.fs.write_file(&self.construct_path(path)?, data).await
    }

    pub async fn remove_item(&self, path: &Path) -> FSRes<()> {
        self.fs.remove_item(&self.construct_path(path)?).await
    }

    pub async fn move_item(&self, source: &Path, target: &Path) -> FSRes<()> {
        self.fs.move_item(
            &self.construct_path(source)?,
            &self.construct_path(target)?
        ).await
    }

    pub async fn copy_item(&self, source: &Path, target: &Path) -> FSRes<()> {
        if let Some(total_size) = self.fs.userspace_size() {
            if self.get_item_size(".".as_ref()).await? + self.get_item_size(source).await? > total_size {
                return Err(FSError::NotEnoughStorage);
            }
        };
        
        self.fs.copy_item(
            &self.construct_path(source)?,
            &self.construct_path(target)?
        ).await
    }

    pub async fn read_file(&self, path: &Path) -> FSRes<Vec<u8>> {
        self.fs.read_file(&self.construct_path(path)?).await
    }

    pub async fn get_item_size(&self, path: &Path) -> FSRes<u64> {
        self.fs.get_item_size(&self.construct_path(path)?).await
    }

    /// returns: (created, modified)
    pub async fn get_item_time_info(&self, path: &Path) -> FSRes<(SystemTime, SystemTime)> {
        self.fs.get_item_time_info(&self.construct_path(path)?).await
    }

    pub async fn get_mime(&self, path: &Path) -> FSRes<Option<String>> {
        self.fs.get_mime(&self.construct_path(path)?).await
    }

    pub async fn get_dir_tree(&self, path: &Path) -> FSRes<Vec<PathBuf>> {
        self.adapt_paths(self.fs.get_dir_tree(&self.construct_path(path)?).await?).await
    }

    pub async fn deploy_template(&self) -> Option<FSRes<()>> {
        Some(self.fs.copy_item(
            self.fs.template_path.as_ref()?,
            &self.base_path,
        ).await)
    }

    /// WARNING: EXPECTS AN ALREADY CONSTRUCTED PATH
    pub fn is_breaking_out(&self, final_path: &Path) -> bool {
        !final_path.starts_with(&self.base_path)
    }

    fn construct_path(&self, path: &Path) -> FSRes<PathBuf> {
        let final_path = self.base_path.join(path).normalize();
        
        if self.is_breaking_out(&final_path) {
            return Err(FSError::PathBreaksOut)
        };

        Ok(final_path)
    }

    async fn adapt_paths(&self, paths: Vec<PathBuf>) -> FSRes<Vec<PathBuf>> {
        let true_base_path = self.fs.construct_path(&self.construct_path(".".as_ref())?)?;
        Ok(paths.into_iter().map(|p| p.strip_prefix(&true_base_path).unwrap().to_path_buf()).collect())
    }
}
