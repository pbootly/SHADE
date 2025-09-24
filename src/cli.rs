use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "shade")]
#[command(about = "Simple Host Attestation & Dynamic Enrollment")]
pub struct Cli {
    #[arg(short, long, default_value = "shade.yaml")]
    config: String,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    GenKeys,
    Server,
    RegisterKey {
        #[arg(short, long)]
        private_key: String,
    },
    RevokeCert {
        #[arg(short, long)]
        id: String,
    },
    ListCerts,
    Validate,
}

pub fn run_cli() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::GenKeys) => {
            let (priv_b64, pub_b64) = crate::cert::generate_keys()?;
            println!("Private key: {}", priv_b64);
            println!("Public key:  {}", pub_b64);
        }
        Some(Commands::Server) => {
            tokio::runtime::Runtime::new()?.block_on(crate::server::run_server(&cli.config))?;
        }
        Some(Commands::RegisterKey { private_key }) => {
            tokio::runtime::Runtime::new()?.block_on(register_key(&cli.config, private_key))?;
        }
        Some(Commands::RevokeCert { id }) => {
            tokio::runtime::Runtime::new()?.block_on(revoke_certificate(&cli.config, id))?;
        }
        Some(Commands::ListCerts) => {
            tokio::runtime::Runtime::new()?.block_on(list_certificates(&cli.config))?;
        }
        Some(Commands::Validate) => {
            let config = crate::config::Config::load(&cli.config)?;
            config.validate()?;
        }
        None => {
            println!("No command provided. Use --help to see available commands.");
        }
    }

    Ok(())
}

async fn register_key(config_path: &str, private_key: String) -> Result<()> {
    let config = crate::config::Config::load(config_path)?;
    config.validate()?;

    match config.storage.mode {
        crate::config::StorageMode::File => {
            let storage = create_storage(&config).await?;
            let keypair = crate::storage::KeyPair::new(private_key, None)?;
            storage.register_key(keypair.clone()).await?;
            println!("Key registered successfully with ID: {}", keypair.id);
        }
        crate::config::StorageMode::Socket => {
            let socket_path = config.storage.socket_path.as_ref().unwrap();
            let client = crate::socket::SocketClient::new(socket_path);
            let keypair = crate::storage::KeyPair::new(private_key, None)?;
            let response = client
                .send_message(crate::socket::SocketMessage::RegisterKey(keypair.clone()))
                .await?;
            match response {
                crate::socket::SocketResponse::KeyRegistered(kp) => {
                    println!("Key registered successfully with ID: {}", kp.id);
                }
                crate::socket::SocketResponse::Error(e) => {
                    anyhow::bail!("Server error: {}", e);
                }
                _ => {
                    anyhow::bail!("Unexpected response from server");
                }
            }
        }
    }
    Ok(())
}

async fn revoke_certificate(config_path: &str, id: String) -> Result<()> {
    let config = crate::config::Config::load(config_path)?;
    config.validate()?;

    match config.storage.mode {
        crate::config::StorageMode::File => {
            let storage = create_storage(&config).await?;
            let uuid = uuid::Uuid::parse_str(&id)?;
            storage.revoke_key(uuid).await?;
            println!("Key with ID {} revoked successfully", id);
        }
        crate::config::StorageMode::Socket => {
            let socket_path = config.storage.socket_path.as_ref().unwrap();
            let client = crate::socket::SocketClient::new(socket_path);
            let response = client
                .send_message(crate::socket::SocketMessage::RevokeKey { id: id.clone() })
                .await?;
            match response {
                crate::socket::SocketResponse::KeyRevoked => {
                    println!("Key with ID {} revoked successfully", id);
                }
                crate::socket::SocketResponse::Error(e) => {
                    anyhow::bail!("Server error: {}", e);
                }
                _ => {
                    anyhow::bail!("Unexpected response from server");
                }
            }
        }
    }

    Ok(())
}
async fn list_certificates(config_path: &str) -> Result<()> {
    let config = crate::config::Config::load(config_path)?;
    config.validate()?;

    match config.storage.mode {
        crate::config::StorageMode::File => {
            let storage = create_storage(&config).await?;
            let keys = storage.list_keys().await?;
            for key in keys {
                println!(
                    "ID: {}, Created At: {}, Expires At: {:?}",
                    key.id, key.created_at, key.expires_at
                );
            }
        }
        crate::config::StorageMode::Socket => {
            let socket_path = config.storage.socket_path.as_ref().unwrap();
            let client = crate::socket::SocketClient::new(socket_path);
            let response = client
                .send_message(crate::socket::SocketMessage::ListKey)
                .await?;
            match response {
                crate::socket::SocketResponse::KeyList(keys) => {
                    for key in keys {
                        println!(
                            "ID: {}, PubKey: {},  Created At: {}, Expires At: {:?}",
                            key.id, key.public_key, key.created_at, key.expires_at
                        );
                    }
                }
                crate::socket::SocketResponse::Error(e) => {
                    anyhow::bail!("Server error: {}", e);
                }
                _ => {
                    anyhow::bail!("Unexpected response from server");
                }
            }
        }
    }

    Ok(())
}

async fn create_storage(
    config: &crate::config::Config,
) -> Result<Box<dyn crate::storage::StorageBackend>> {
    let database_url = config.storage.database_url.as_ref().unwrap();
    let storage = crate::storage::SqliteStorage::new(database_url).await?;
    Ok(Box::new(storage))
}
