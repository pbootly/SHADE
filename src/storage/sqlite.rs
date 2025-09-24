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
        let pool = SqlitePool::connect(database_url).await?;

        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl StorageBackend for SqliteStorage {
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
