use std::fs::Metadata;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::SystemTime;
use normalize_path::NormalizePath;

mod user_scope;


pub use user_scope::UserScopedFS;


#[derive(Debug)]
pub struct Filesystem {
    storage_path: PathBuf,
    template_path: Option<PathBuf>,
    total_size: Option<u64>,
    userspace_size: Option<u64>,
}


#[derive(Debug)]
pub enum FSError {
    HFS(std::io::Error),
    PathBreaksOut,
    InvalidUTF8Path,
    NotEnoughStorage
}


impl std::fmt::Display for FSError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HFS(hfs_err) => write!(f, "a host fs error occured: {hfs_err}"),
            Self::PathBreaksOut => write!(f, "the path is invalid"),
            Self::InvalidUTF8Path => write!(f, "the path is not valid a UTF-8 string"),
            Self::NotEnoughStorage => write!(f, "you haven't got enough storage to store a file of such size"),
        }
    }
}


pub type FSRes<T> = Result<T, FSError>;


impl Filesystem {
    pub fn new(storage_path: &Path, template_path: Option<&Path>, total_size: Option<u64>, userspace_size: Option<u64>) -> Self {
        log::debug!("initializing fs...");
        
        if !storage_path.exists() {
            std::fs::create_dir(storage_path).unwrap()
        } else if !storage_path.is_dir() {
            panic!("filesystem's storage path must be a path to a directory")
        };
        
        Self {
            userspace_size, total_size,
            storage_path: storage_path.canonicalize().unwrap(),
            template_path: template_path.map(|p| {
                if !p.is_dir() {
                    panic!("template path must be a path to an existing directory")
                };

                // todo remove such requirement
                // todo check if p is under and relative to storage_path as well
                
                p.to_path_buf()
            }),
        }
    }
    
    pub fn storage_path(&self) -> &Path {
        &self.storage_path
    }
    
    pub fn template_path(&self) -> Option<&Path> {
        self.template_path.as_deref()
    }
    
    pub fn userspace_size(&self) -> Option<u64> {
        self.userspace_size
    }
    
    pub fn total_size(&self) -> Option<u64> {
        self.total_size
    }

    pub async fn create_dir(&self, path: &Path) -> FSRes<()> {
        tokio::fs::create_dir(self.construct_path(path)?).await.map_err(FSError::HFS)?;
        Ok(())
    }
    
    pub async fn list_dir(&self, path: &Path) -> FSRes<(Vec<PathBuf>, Vec<PathBuf>)> {
        let mut files = Vec::new();
        let mut directories = Vec::new();
        
        let mut dir_iter = tokio::fs::read_dir(self.construct_path(path)?).await.map_err(FSError::HFS)?;
        while let Some(item) = dir_iter.next_entry().await.map_err(FSError::HFS)? {
            let item_path = item.path();
            
            if item_path.is_dir() {
                directories.push(item_path);
            } else {
                files.push(item_path);
            };
        };
        
        Ok((files, directories))
    }
    
    pub async fn write_file(&self, path: &Path, data: &[u8]) -> FSRes<()> {
        if let Some(total_size) = self.total_size {
            if self.get_item_size(".".as_ref()).await? + data.len() as u64 > total_size {
                return Err(FSError::NotEnoughStorage);
            }
        };
        
        tokio::fs::write(self.construct_path(path)?, data).await.map_err(FSError::HFS)?;
        Ok(())
    }

    pub async fn remove_item(&self, path: &Path) -> FSRes<()> {
        let path = self.construct_path(path)?;

        if path.is_file() {
            tokio::fs::remove_file(path).await.map_err(FSError::HFS)?;
        } else {
            tokio::fs::remove_dir_all(path).await.map_err(FSError::HFS)?;
        };

        Ok(())
    }

    pub async fn move_item(&self, source: &Path, target: &Path) -> FSRes<()> {
        tokio::fs::rename(
            self.construct_path(source)?,
            self.construct_path(target)?
        ).await.map_err(FSError::HFS)?;
        Ok(())
    }

    pub async fn copy_item(&self, source: &Path, target: &Path) -> FSRes<()> {
        let source = self.construct_path(source)?;

        if let Some(total_size) = self.total_size {
            if self.get_item_size(".".as_ref()).await? + self.get_item_size(&source).await? > total_size {
                return Err(FSError::NotEnoughStorage);
            }
        };

        let target = self.construct_path(target)?;

        if source.is_file() {
            tokio::fs::copy(source, target).await.map_err(FSError::HFS)?;
        } else {
            let mut source = source;

            source.push("**");
            source.push("*");

            let base_path = self.storage_path.clone();  // xxx is there really not a better solution?
            tokio::task::spawn_blocking(move || {
                for item in glob::glob(source.to_str().ok_or(FSError::InvalidUTF8Path)?).unwrap().filter_map(Result::ok) {
                    let target = target.join(item.strip_prefix(&base_path).unwrap());

                    if item.is_file() {
                        let parent = target.parent().unwrap();
                        if !parent.exists() {
                            std::fs::create_dir_all(parent).unwrap();
                        };

                        std::fs::copy(item, target).map_err(FSError::HFS)?;
                    } else {
                        std::fs::create_dir_all(target).unwrap();
                    };
                };

                Ok(())
            }).await.unwrap()?;
        };

        Ok(())
    }

    pub async fn read_file(&self, path: &Path) -> FSRes<Vec<u8>> {
        tokio::fs::read(self.construct_path(path)?).await.map_err(FSError::HFS)
    }

    pub async fn get_item_size(&self, path: &Path) -> FSRes<u64> {
        let path = self.construct_path(path)?;

        if path.is_file() {
            Ok(tokio::fs::metadata(path).await.map_err(FSError::HFS)?.len())
        } else {
            tokio::task::spawn_blocking(move || {
                let mut path = path;

                path.push("**");
                path.push("*");

                Ok(glob::glob(path.to_str().ok_or(FSError::InvalidUTF8Path)?).unwrap()
                    .filter_map(|p| p.ok())
                    .fold(0, |total, p| total + p.metadata().unwrap().len()))
            }).await.unwrap()
        }
    }

    /// returns: (created, modified)
    pub async fn get_item_time_info(&self, path: &Path) -> FSRes<(SystemTime, SystemTime)> {  // xxx or should it be chrono::DateTime?
        let metadata = self.get_item_metadata(&self.construct_path(path)?).await?;

        Ok((
            metadata.created().map_err(FSError::HFS)?,
            metadata.modified().map_err(FSError::HFS)?
        ))
    }

    pub async fn get_mime(&self, path: &Path) -> FSRes<Option<String>> {
        Ok(mime_guess::from_path(self.construct_path(path)?).first().map(|mm| mm.to_string()))
    }
    
    pub async fn get_dir_tree(&self, path: &Path) -> FSRes<Vec<PathBuf>> {
        let mut path = self.construct_path(path)?;

        path.push("**");
        path.push("*");

        tokio::task::spawn_blocking(move || {
            Ok(glob::glob(path.to_str().ok_or(FSError::InvalidUTF8Path)?).unwrap()
                .filter_map(|p| p.ok())
                .collect())
        }).await.unwrap()
    }

    /// WARNING: EXPECTS AN ALREADY CONSTRUCTED PATH
    pub fn is_breaking_out(&self, final_path: &Path) -> bool {
        !final_path.starts_with(&self.storage_path)
    }

    fn construct_path(&self, path: &Path) -> FSRes<PathBuf> {
        let final_path = self.storage_path.join(path).normalize();

        if self.is_breaking_out(&final_path) {
            return Err(FSError::PathBreaksOut)
        };

        Ok(final_path)
    }

    // xxx should it be public?
    async fn get_item_metadata(&self, path: &Path) -> FSRes<Metadata> {
        tokio::fs::metadata(self.construct_path(path)?).await.map_err(FSError::HFS)
    }
}
