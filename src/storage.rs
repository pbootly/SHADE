use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPair {
    pub id: Uuid,
    pub public_key: String,
    pub private_key: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl KeyPair {
    pub fn new(private_key: String, expires_at: Option<DateTime<Utc>>) -> anyhow::Result<Self> {
        let public_key = crate::cert::generate_public_from_private(&private_key)?;
        Ok(Self {
            id: Uuid::new_v4(),
            private_key,
            public_key,
            created_at: Utc::now(),
            expires_at,
        })
    }
}

#[async_trait]
pub trait StorageBackend: Send + Sync + Debug {
    async fn register_key(&self, keypair: KeyPair) -> Result<()>;
    async fn revoke_key(&self, id: Uuid) -> Result<()>;
    async fn list_keys(&self) -> Result<Vec<KeyPair>>;
#[allow(dead_code)]
    async fn validate_public_key(&self, public_key: &str) -> Result<bool>;
    #[allow(dead_code)]
    async fn store_client_ip(&self, ip_address: String) -> Result<()>;
}

pub mod sqlite;

pub use sqlite::SqliteStorage;
