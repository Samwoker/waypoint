use serde::Deserialize;

use crate::error::CoreError;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub data_encryption_key: String,
    pub jwt_secret: String,
    #[serde(default = "default_api_port")]
    pub api_port: u16,
    #[serde(default = "default_environment")]
    pub environment: String,
}

fn default_api_port() -> u16 {
    3000
}

fn default_environment() -> String {
    "development".to_string()
}

impl Config {
    pub fn load() -> Result<Self, CoreError> {
        let _ = dotenvy::dotenv();

        let cfg = config::Config::builder()
            .add_source(config::Environment::default())
            .build()
            .map_err(|e| CoreError::Internal(format!("Failed to build config: {e}")))?;

        cfg.try_deserialize::<Self>()
            .map_err(|e| CoreError::Internal(format!("Failed to deserialize config: {e}")))
    }
}
