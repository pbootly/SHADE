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
    pub fn load_from_path(config_path: &str) -> Result<Self> {
        if Path::new(config_path).exists() {
            let content = fs::read_to_string(config_path)?;
            let config: Config = serde_yaml::from_str(&content)?;
            Ok(config)
        } else {
            // Create default config file
            let config = Config::default();
            let yaml = serde_yaml::to_string(&config)?;
            fs::write(config_path, yaml)?;
            println!("Created default config file: {}", config_path);
            Ok(config)
        }
    }

    pub fn validate(&self) -> Result<()> {
        // Security guard: SQLite only allowed on localhost
        if self.server.host != "127.0.0.1" && self.server.host != "localhost" {
            anyhow::bail!(
                "SQLite storage is only allowed when server is bound to 127.0.0.1 or localhost. Current host: {}",
                self.server.host
            );
        }

        // Validate storage configuration based on mode
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
