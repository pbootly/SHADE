use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tokio::net::{UnixListener, UnixStream};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SocketMessage {
    Register(crate::storage::KeyPair),
    Revoke { id: String },
    List,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SocketResponse {
    KeyRegistered(crate::storage::KeyPair),
    KeyRevoked,
    KeyList(Vec<crate::storage::KeyPair>),
    Error(String),
}

pub struct SocketServer {
    listener: UnixListener,
    storage: Arc<dyn crate::storage::StorageBackend>,
}

impl SocketServer {
    pub async fn new(
        socket_path: &str,
        storage: Arc<dyn crate::storage::StorageBackend>,
    ) -> Result<Self> {
        if Path::new(socket_path).exists() {
            std::fs::remove_file(socket_path)?;
        }

        let listener = UnixListener::bind(socket_path)?;
        Ok(Self { listener, storage })
    }

    pub async fn run(&self) -> Result<()> {
        println!(
            "Socket server listening on {}",
            self.listener
                .local_addr()?
                .as_pathname()
                .unwrap()
                .to_string_lossy()
        );

        while let Ok((stream, _addr)) = self.listener.accept().await {
            let storage = self.storage.clone();
            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(stream, storage).await {
                    eprintln!("Error handling connection: {}", e);
                }
            });
        }

        Ok(())
    }

    async fn handle_connection(
        stream: UnixStream,
        storage: Arc<dyn crate::storage::StorageBackend>,
    ) -> Result<()> {
        let mut framed = Framed::new(stream, LengthDelimitedCodec::new());

        while let Some(frame) = framed.next().await {
            let frame = frame?;
            let message: SocketMessage = serde_json::from_slice(&frame)?;

            let response = match message {
                SocketMessage::Register(kp) => match storage.register_key(kp.clone()).await {
                    Ok(_) => SocketResponse::KeyRegistered(kp),
                    Err(e) => SocketResponse::Error(e.to_string()),
                },
                SocketMessage::Revoke { id } => match uuid::Uuid::parse_str(&id) {
                    Ok(uuid) => match storage.revoke_key(uuid).await {
                        Ok(_) => SocketResponse::KeyRevoked,
                        Err(e) => SocketResponse::Error(e.to_string()),
                    },
                    Err(e) => SocketResponse::Error(e.to_string()),
                },
                SocketMessage::List => match storage.list_keys().await {
                    Ok(keys) => SocketResponse::KeyList(keys),
                    Err(e) => SocketResponse::Error(e.to_string()),
                },
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

        let response_frame = framed
            .next()
            .await
            .ok_or_else(|| anyhow::anyhow!("No response received"))??;
        let response: SocketResponse = serde_json::from_slice(&response_frame)?;

        Ok(response)
    }
}
