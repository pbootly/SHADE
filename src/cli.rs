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
    RegisterCert {
        // TODO: Implement passing public and private
    },
    RevokeCert {
        #[arg(short, long)]
        id: String,
    },
    /// List all certificates
    ListCerts,
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
        Some(Commands::RegisterCert {
            // TODO: Public and Private
        }) => {
                    }
        Some(Commands::RevokeCert { id }) => {
            tokio::runtime::Runtime::new()?.block_on(revoke_certificate(&cli.config, id))?;
        }
        Some(Commands::ListCerts) => {
            tokio::runtime::Runtime::new()?.block_on(list_certificates(&cli.config))?;
        }
        None => {
            println!("No command provided. Use --help to see available commands.");
        }
    }

    Ok(())
}

//async fn register_certificate() -> Result<()> {
//    let config = crate::config::Config::load_from_path(config_path)?;
//    config.validate()?;
//
//    let cert_data = std::fs::read_to_string(&cert_file)?;
//    let expires_at = if let Some(expires_str) = expires_at {
//        Some(chrono::DateTime::parse_from_rfc3339(&expires_str)?.with_timezone(&chrono::Utc))
//    } else {
//        None
//    };
//
//    let cert_registration = crate::storage::CertificateRegistration {
//        certificate_data: cert_data,
//        expires_at,
//        subject,
//        issuer,
//        serial_number: serial,
//    };
//
//    match config.storage.mode {
//        crate::config::StorageMode::File => {
//            let storage = create_storage(&config).await?;
//            let certificate = storage.register_certificate(cert_registration).await?;
//            print_certificate_info(&certificate);
//        }
//        crate::config::StorageMode::Socket => {
//            let socket_path = config.storage.socket_path.as_ref().unwrap();
//            let client = crate::socket::SocketClient::new(socket_path);
//            let response = client
//                .send_message(crate::socket::SocketMessage::RegisterCert(
//                    cert_registration,
//                ))
//                .await?;
//            match response {
//                crate::socket::SocketResponse::CertRegistered(certificate) => {
//                    print_certificate_info(&certificate);
//                }
//                crate::socket::SocketResponse::Error(e) => {
//                    anyhow::bail!("Server error: {}", e);
//                }
//                _ => {
//                    anyhow::bail!("Unexpected response from server");
//                }
//            }
//        }
//    }
//
//    Ok(())
//}

async fn revoke_certificate(config_path: &str, id: String) -> Result<()> {
    let config = crate::config::Config::load_from_path(config_path)?;
    config.validate()?;

    match config.storage.mode {
        crate::config::StorageMode::File => {
            let storage = create_storage(&config).await?;
            let cert_id = uuid::Uuid::parse_str(&id)?;
            storage.revoke_certificate(cert_id).await?;
            println!("Certificate {} revoked successfully", id);
        }
        crate::config::StorageMode::Socket => {
            let socket_path = config.storage.socket_path.as_ref().unwrap();
            let client = crate::socket::SocketClient::new(socket_path);
            let response = client
                .send_message(crate::socket::SocketMessage::RevokeCert { id: id.clone() })
                .await?;
            match response {
                crate::socket::SocketResponse::CertRevoked => {
                    println!("Certificate {} revoked successfully", id);
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

// TODO: print certificates
async fn list_certificates(config_path: &str) -> Result<()> {
    let config = crate::config::Config::load_from_path(config_path)?;
    config.validate()?;

    match config.storage.mode {
        crate::config::StorageMode::File => {
            let storage = create_storage(&config).await?;
            let certificates = storage.list_certificates().await?;
        }
        crate::config::StorageMode::Socket => {
            let socket_path = config.storage.socket_path.as_ref().unwrap();
            let client = crate::socket::SocketClient::new(socket_path);
            let response = client
                .send_message(crate::socket::SocketMessage::ListCerts)
                .await?;
            match response {
                crate::socket::SocketResponse::CertList(certificates) => {}
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
