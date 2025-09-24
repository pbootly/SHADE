use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub storage: StorageConfig,
    pub server: ServerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub mode: StorageMode,
    pub database_url: Option<String>,
    pub socket_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageMode {
    #[serde(rename = "file")]
    File,
    #[serde(rename = "socket")]
    Socket,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            storage: StorageConfig {
                mode: StorageMode::Socket,
                database_url: Some("sqlite::memory:".to_string()),
                socket_path: Some("/tmp/shade.sock".to_string()),
            },
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 3000,
            },
        }
    }
}

impl Config {
    pub fn load(config_path: &str) -> Result<Self> {
        if Path::new(config_path).exists() {
            return Self::load_from_path(config_path);
        }
        println!(
            "Warning: Config file '{}' not found. Using default configuration.",
            config_path
        );
        Ok(Config::default())
    }

    pub fn load_from_path(config_path: &str) -> Result<Self> {
        let content = fs::read_to_string(config_path)?;
        let config: Config = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<()> {
        match self.storage.mode {
            StorageMode::File => {
                if self.storage.database_url.is_none() {
                    anyhow::bail!("database_url is required for file mode");
                }
            }
            StorageMode::Socket => {
                if self.storage.database_url.is_none() {
                    anyhow::bail!("database_url is required for socket mode");
                }
                if self.storage.socket_path.is_none() {
                    anyhow::bail!("socket_path is required for socket mode");
                }
            }
        }

        Ok(())
    }
}
