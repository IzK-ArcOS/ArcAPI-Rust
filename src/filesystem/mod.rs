use std::path::{Path, PathBuf};
use std::time::SystemTime;

mod user_scope;


pub struct Filesystem {
    base_path: PathBuf,
    template_path: Option<PathBuf>,
    userspace_size: Option<u64>,
}


pub enum FSError {
    HFS(std::io::Error),
    PathBreaksOut,
    InvalidUTF8Path
}


type FSRes<T> = Result<T, FSError>;


impl Filesystem {
    pub fn new(base_path: &Path, template_path: Option<&Path>, userspace_size: Option<u64>) -> Self {
        Self {
            userspace_size,
            base_path: base_path.canonicalize().unwrap(),
            template_path: template_path.map(|p| p.canonicalize().unwrap()),
        }
    }
    
    pub fn base_path(&self) -> &Path {
        &self.base_path
    }
    
    pub fn template_path(&self) -> Option<&Path> {
        self.template_path.as_deref()
    }
    
    pub fn userspace_size(&self) -> Option<u64> {
        self.userspace_size
    }

    pub async fn create_dir(&self, path: &Path) -> FSRes<()> {
        tokio::fs::create_dir(self.construct_path(path).await?).await.map_err(FSError::HFS)?;
        Ok(())
    }
    
    pub async fn list_dir(&self, path: &Path) -> FSRes<(Vec<PathBuf>, Vec<PathBuf>)> {
        let mut files = Vec::new();
        let mut directories = Vec::new();
        
        let mut dir_iter = tokio::fs::read_dir(self.construct_path(path).await?).await.map_err(FSError::HFS)?;
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
        tokio::fs::write(self.construct_path(path).await?, data).await.map_err(FSError::HFS)?;
        Ok(())
    }

    pub async fn remove_item(&self, path: &Path) -> FSRes<()> {
        let path = self.construct_path(path).await?;

        if path.is_file() {
            tokio::fs::remove_file(path).await.map_err(FSError::HFS)?;
        } else {
            tokio::fs::remove_dir_all(path).await.map_err(FSError::HFS)?;
        };

        Ok(())
    }

    pub async fn move_item(&self, source: &Path, target: &Path) -> FSRes<()> {
        tokio::fs::rename(
            self.construct_path(source).await?,
            self.construct_path(target).await?
        ).await.map_err(FSError::HFS)?;
        Ok(())
    }

    pub async fn copy_item(&self, source: &Path, target: &Path) -> FSRes<()> {
        let source = self.construct_path(source).await?;
        let target = self.construct_path(target).await?;

        if source.is_file() {
            tokio::fs::copy(source, target).await.map_err(FSError::HFS)?;
        } else {
            let mut source = source;

            source.push("**");
            source.push("*");

            let base_path = self.base_path.clone();  // xxx is there really not a better solution?
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
        tokio::fs::read(self.construct_path(path).await?).await.map_err(FSError::HFS)
    }

    pub async fn get_item_size(&self, path: &Path) -> FSRes<u64> {
        let path = self.construct_path(path).await?;

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
    
    pub async fn get_item_creation_time(&self, path: &Path) -> FSRes<SystemTime> { 
        tokio::fs::metadata(self.construct_path(path).await?).await.map_err(FSError::HFS)?
            .created().map_err(FSError::HFS)
    }

    pub async fn get_mime(&self, path: &Path) -> FSRes<Option<String>> {
        Ok(mime_guess::from_path(self.construct_path(path).await?).first().map(|mm| mm.to_string()))
    }
    
    pub async fn get_tree(&self, path: &Path) -> FSRes<Vec<PathBuf>> {
        let mut path = self.construct_path(path).await?;
        path.push("**");
        path.push("*");
        tokio::task::spawn_blocking(move || {
            Ok(glob::glob(path.to_str().ok_or(FSError::InvalidUTF8Path)?).unwrap()
                .filter_map(|p| p.ok())
                .collect())
        }).await.unwrap()
    }

    async fn is_breaking_out(&self, path: &Path) -> FSRes<bool> {
        let path = tokio::fs::canonicalize(path).await.map_err(FSError::HFS)?;

        Ok(!path.starts_with(&self.base_path))
    }

    async fn construct_path(&self, path: &Path) -> FSRes<PathBuf> {
        let c_path = self.base_path.join(path);

        if self.is_breaking_out(&c_path).await? {
            return Err(FSError::PathBreaksOut)
        };

        Ok(c_path)
    }
}
