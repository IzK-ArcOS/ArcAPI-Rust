use std::env::VarError;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub address: String
}

#[derive(Debug, Deserialize)]
struct PartialDBConfig {
    pub conn_pool_size: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct PartialConfig {
    pub platform: String,
    pub server: ServerConfig,
    pub database: PartialDBConfig,
}


#[derive(Debug, Deserialize)]
pub struct AuthConfig {
    pub code: Option<String>,
}


#[derive(Debug)]
pub struct DBConfig {
    pub path: String,
    pub conn_pool_size: u32,
}


#[derive(Debug)]
pub struct Config {
    pub platform: String,
    pub server: ServerConfig,
    pub database: DBConfig,
    pub auth: AuthConfig,
}


impl Config {
    pub const DEFAULT_CONN_POOL_SIZE: u32 = 16;
    pub const CONFIG_FILE_PATH_ENV_VAR: &'static str = "CONFIG_FILE";
    pub const DATABASE_FILE_PATH_ENV_VAR: &'static str = "DATABASE_URL";
    pub const AUTH_CODE_ENV_VAR: &'static str = "AUTH_CODE";
    
    pub fn load() -> Self {
        let path = get_env_var(Self::CONFIG_FILE_PATH_ENV_VAR);
        
        let config_raw = std::fs::read_to_string(&path)
            .expect(&format!("{path} should be a valid config file"));

        let part = toml::from_str::<PartialConfig>(&config_raw)
            .expect(&format!("{path} should be a valid config file"));

        Self {
            platform: part.platform,
            server: part.server,
            database: DBConfig {
                path: get_env_var(Self::DATABASE_FILE_PATH_ENV_VAR),
                conn_pool_size: part.database.conn_pool_size.unwrap_or(Self::DEFAULT_CONN_POOL_SIZE)
            },
            auth: AuthConfig {
                code: get_opt_env_var(Self::AUTH_CODE_ENV_VAR)
            }
        }
    }
}


fn get_env_var(name: &str) -> String {
    std::env::var(name)
        .expect(&format!("env var '{name}' should be set and valid"))
}


fn get_opt_env_var(name: &str) -> Option<String> {
    match std::env::var(name) {
        Ok(v) => Some(v),
        Err(VarError::NotPresent) => None,
        err @ Err(_) => { err.unwrap(); unreachable!("this is an error case, unwrap always will fail") } 
    }
}
