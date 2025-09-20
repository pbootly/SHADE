use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::net::{UnixListener, UnixStream};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SocketMessage {
    RegisterCert(crate::storage::CertificateRegistration),
    RevokeCert { id: String },
    ListCerts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SocketResponse {
    CertRegistered(crate::storage::Certificate),
    CertRevoked,
    CertList(Vec<crate::storage::Certificate>),
    Error(String),
}

pub struct SocketServer {
    listener: UnixListener,
    storage: std::sync::Arc<Box<dyn crate::storage::StorageBackend>>,
}

impl SocketServer {
    pub async fn new(socket_path: &str, storage: std::sync::Arc<Box<dyn crate::storage::StorageBackend>>) -> Result<Self> {
        // Remove existing socket file if it exists
        if Path::new(socket_path).exists() {
            std::fs::remove_file(socket_path)?;
        }

        let listener = UnixListener::bind(socket_path)?;
        Ok(Self { listener, storage })
    }

    pub async fn run(&self) -> Result<()> {
        println!("Socket server listening on {}", self.listener.local_addr()?.as_pathname().unwrap().to_string_lossy());

        while let Some(stream) = self.listener.accept().await.ok() {
            let storage = self.storage.clone();
            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(stream.0, storage).await {
                    eprintln!("Error handling connection: {}", e);
                }
            });
        }

        Ok(())
    }

    async fn handle_connection(stream: UnixStream, storage: std::sync::Arc<Box<dyn crate::storage::StorageBackend>>) -> Result<()> {
        let mut framed = Framed::new(stream, LengthDelimitedCodec::new());

        while let Some(frame) = framed.next().await {
            let frame = frame?;
            let message: SocketMessage = serde_json::from_slice(&frame)?;

            let response = match message {
                SocketMessage::RegisterCert(cert_reg) => {
                    match storage.register_certificate(cert_reg).await {
                        Ok(cert) => SocketResponse::CertRegistered(cert),
                        Err(e) => SocketResponse::Error(e.to_string()),
                    }
                }
                SocketMessage::RevokeCert { id } => {
                    match uuid::Uuid::parse_str(&id) {
                        Ok(uuid) => {
                            match storage.revoke_certificate(uuid).await {
                                Ok(_) => SocketResponse::CertRevoked,
                                Err(e) => SocketResponse::Error(e.to_string()),
                            }
                        }
                        Err(e) => SocketResponse::Error(e.to_string()),
                    }
                }
                SocketMessage::ListCerts => {
                    match storage.list_certificates().await {
                        Ok(certs) => SocketResponse::CertList(certs),
                        Err(e) => SocketResponse::Error(e.to_string()),
                    }
                }
            };

            let response_bytes = serde_json::to_vec(&response)?;
            framed.send(response_bytes.into()).await?;
        }

        Ok(())
    }
}

pub struct SocketClient {
    socket_path: String,
}

impl SocketClient {
    pub fn new(socket_path: &str) -> Self {
        Self {
            socket_path: socket_path.to_string(),
        }
    }

    pub async fn send_message(&self, message: SocketMessage) -> Result<SocketResponse> {
        let stream = UnixStream::connect(&self.socket_path).await?;
        let mut framed = Framed::new(stream, LengthDelimitedCodec::new());

        let message_bytes = serde_json::to_vec(&message)?;
        framed.send(message_bytes.into()).await?;

        let response = framed.next().await
            .ok_or_else(|| anyhow::anyhow!("No response received"))??;
        let response: SocketResponse = serde_json::from_slice(&response)?;

        Ok(response)
    }
}