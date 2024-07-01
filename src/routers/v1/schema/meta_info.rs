use serde::{Deserialize, Serialize};
use crate::config::Config;

#[derive(Serialize, Deserialize)]
pub struct MetaInfo {
    platform: String,
    port: u16,
    referrer: String,
    valid: bool,
    revision: u32,
    protected: bool,
}


impl From<&Config> for MetaInfo {
    fn from(cfg: &Config) -> Self {
        Self {
            platform: cfg.platform.clone(),
            port: cfg.server.port,
            referrer: "/connect".to_string(),
            valid: true,
            revision: 2,
            protected: cfg.auth.code.is_some()
        }
    }
}
