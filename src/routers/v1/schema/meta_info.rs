use serde::{Deserialize, Serialize};
use crate::config::Config;

#[derive(Serialize, Deserialize)]
pub struct MetaInfo {
    pub platform: String,
    pub port: u16,
    pub referrer: String,
    pub revision: u32,
    pub protected: bool,
}


impl MetaInfo {
    pub fn new(cfg: &Config) -> Self {
        Self {
            platform: cfg.name.clone(),
            port: cfg.server.port,
            referrer: "/connect".to_string(),
            revision: 2,
            protected: cfg.auth.code.is_some()
        }
    }
}
