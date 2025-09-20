use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certificate {
    pub id: Uuid,
    pub certificate_data: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub subject: String,
    pub issuer: String,
    pub serial_number: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateRegistration {
    pub certificate_data: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub subject: String,
    pub issuer: String,
    pub serial_number: String,
}

#[async_trait]
pub trait StorageBackend: Send + Sync + Debug {
    async fn register_certificate(&self, cert: CertificateRegistration) -> Result<Certificate>;
    async fn revoke_certificate(&self, id: Uuid) -> Result<()>;
    async fn list_certificates(&self) -> Result<Vec<Certificate>>;
    // TODO: implementation of StorageBackend for SQLite is missing these methods
    #[allow(dead_code)]
    async fn get_certificate(&self, id: Uuid) -> Result<Option<Certificate>>;
    #[allow(dead_code)]
    async fn health_check(&self) -> Result<()>;
}

pub mod sqlite;

pub use sqlite::SqliteStorage;
