use serde::{Deserialize, Serialize};
use crate::config::Config;

#[derive(Serialize, Deserialize)]
pub struct MetaInfo {
    protected: bool,
    revision: u32,
    name: String,
}


impl From<&Config> for MetaInfo {
    fn from(cfg: &Config) -> Self {
        Self {
            name: cfg.name.clone(),
            revision: 2,
            protected: cfg.auth.code.is_some()
        }
    }
}
