use std::env::VarError;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub address: String
}


#[derive(Debug, Deserialize)]
struct PartialDBConfig {
    pub conn_pool_size: u32,
}


#[derive(Debug, Deserialize)]
struct PartialAuthConfig {
    pub session_lifetime: Option<u64>,
}


#[derive(Debug, Deserialize)]
pub struct FilesystemConfig {
    pub storage_path: PathBuf,
    pub template_path: Option<PathBuf>,
    pub total_size: Option<u64>,
    pub user_space_size: Option<u64>,
} 


#[derive(Debug, Deserialize)]
struct PartialConfig {
    pub name: String,
    pub server: ServerConfig,
    pub filesystem: FilesystemConfig,
    pub database: PartialDBConfig,
    pub auth: PartialAuthConfig,
}


#[derive(Debug, Deserialize)]
pub struct AuthConfig {
    pub code: Option<String>,
    pub session_lifetime: Option<u64>,
}


#[derive(Debug)]
pub struct DBConfig {
    pub path: String,
    pub conn_pool_size: u32,
}


#[derive(Debug)]
pub struct Config {
    pub name: String,
    pub server: ServerConfig,
    pub filesystem: FilesystemConfig,
    pub database: DBConfig,
    pub auth: AuthConfig,
}


impl Config {
    pub const DEFAULT_CONFIG_FILE_CONTENTS: &'static str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/arcapi.default.toml"));
    
    pub const CONFIG_FILE_PATH_ENV_VAR: &'static str = "CONFIG_FILE";
    pub const DATABASE_FILE_PATH_ENV_VAR: &'static str = "DATABASE_URL";
    pub const AUTH_CODE_ENV_VAR: &'static str = "AUTH_CODE";
    
    pub fn load() -> Self {
        let path = get_env_var(Self::CONFIG_FILE_PATH_ENV_VAR);
        
        let config_raw = Self::read(path.as_str().as_ref());

        let part = toml::from_str::<PartialConfig>(&config_raw)
            .unwrap_or_else(|err| panic!("{path} should be a valid config file:\n{err}"));

        Self {
            name: part.name,
            server: part.server,
            filesystem: part.filesystem,
            database: DBConfig {
                path: get_env_var(Self::DATABASE_FILE_PATH_ENV_VAR),
                conn_pool_size: part.database.conn_pool_size
            },
            auth: AuthConfig {
                code: get_opt_env_var(Self::AUTH_CODE_ENV_VAR),
                session_lifetime: part.auth.session_lifetime
            }
        }
    }
    
    fn read(path: &Path) -> String {
        std::fs::read_to_string(&path)
            .unwrap_or_else(|err| match err {
                _ if err.kind() == ErrorKind::NotFound => {
                    Self::create_default(path);
                    Self::read(path)
                },
                err => panic!("unhandled error occurred during config file loading: {err}") 
            })
    }

    fn create_default(path: &Path) {
        std::fs::write(path, Self::DEFAULT_CONFIG_FILE_CONTENTS)
            .unwrap_or_else(|err| panic!("an error occurred during creation of default config file: {err}"));
    }
}


fn get_env_var(name: &str) -> String {
    std::env::var(name)
        .unwrap_or_else(|_| panic!("env var '{name}' should be set and valid"))
}


fn get_opt_env_var(name: &str) -> Option<String> {
    match std::env::var(name) {
        Ok(v) => Some(v),
        Err(VarError::NotPresent) => None,
        err @ Err(_) => { err.unwrap(); unreachable!("this is an error case, unwrap always will fail") } 
    }
}
