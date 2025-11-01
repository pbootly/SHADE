use super::StorageBackend;
use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use sqlx::{Pool, Row, Sqlite, SqlitePool};
use std::fmt::Debug;
use uuid::Uuid;

#[derive(Debug)]
pub struct SqliteStorage {
    pool: Pool<Sqlite>,
}

impl SqliteStorage {
    pub async fn new(database_url: &str) -> Result<Self> {
        if let Some(path) = database_url.strip_prefix("sqlite://")
            && !std::path::Path::new(path).exists()
        {
            std::fs::File::create(path)?;
        }
        let pool = SqlitePool::connect(database_url).await?;
        println!("Running initial DB migration");

        let migration_sql = include_str!("../migrations/001_initial.sql");
        sqlx::query(migration_sql).execute(&pool).await?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl StorageBackend for SqliteStorage {
    async fn validate_public_key(&self, public_key: &str) -> Result<bool> {
        let result = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM keys WHERE public_key = ?)")
            .bind(public_key)
            .fetch_one(&self.pool)
            .await?;
        Ok(result)
    }
    async fn validate_host_ip(&self, ip_address: &str) -> Result<bool> {
        let exists =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM client_ips WHERE ip_address = ?)")
                .bind(ip_address)
                .fetch_one(&self.pool)
                .await?;
        Ok(exists)
    }

    async fn register_key(&self, keypair: super::KeyPair) -> Result<()> {
        let id = keypair.id;
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO keys (id, public_key, private_key, created_at, expires_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(id.to_string())
        .bind(&keypair.public_key)
        .bind(&keypair.private_key)
        .bind(now)
        .bind(keypair.expires_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
    async fn revoke_key(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM keys WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }
    async fn store_client_ip(&self, ip_address: String) -> Result<()> {
        sqlx::query("INSERT INTO client_ips (ip_address, created_at) VALUES (?, ?)")
            .bind(ip_address)
            .bind(chrono::Utc::now())
            .execute(&self.pool)
            .await?;

        Ok(())
    }
    async fn list_hosts(&self) -> Result<Vec<super::HostPair>> {
        let rows = sqlx::query("SELECT ip_address, created_at FROM client_ips")
            .fetch_all(&self.pool)
            .await?;

        let hosts = rows
            .into_iter()
            .map(|row| super::HostPair {
                ip: row.get("ip_address"),
                created_at: row.get("created_at"),
            })
            .collect();

        Ok(hosts)
    }

    async fn list_keys(&self) -> Result<Vec<super::KeyPair>> {
        let rows =
            sqlx::query("SELECT id, public_key, private_key, created_at, expires_at FROM keys")
                .fetch_all(&self.pool)
                .await?;

        let keys = rows
            .into_iter()
            .map(|row| super::KeyPair {
                id: Uuid::parse_str(row.get::<String, _>("id").as_str()).unwrap(),
                public_key: row.get("public_key"),
                private_key: row.get("private_key"),
                created_at: row.get("created_at"),
                expires_at: row.get("expires_at"),
            })
            .collect();

        Ok(keys)
    }
}
