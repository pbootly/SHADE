use super::{Certificate, CertificateRegistration, StorageBackend};
use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use sqlx::{Pool, Sqlite, SqlitePool, Row};
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
    async fn register_certificate(&self, cert: CertificateRegistration) -> Result<Certificate> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        
        sqlx::query(
            r#"
            INSERT INTO certificates (id, certificate_data, created_at, expires_at, subject, issuer, serial_number)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(id.to_string())
        .bind(&cert.certificate_data)
        .bind(now)
        .bind(cert.expires_at)
        .bind(&cert.subject)
        .bind(&cert.issuer)
        .bind(&cert.serial_number)
        .execute(&self.pool)
        .await?;

        Ok(Certificate {
            id,
            certificate_data: cert.certificate_data,
            created_at: now,
            expires_at: cert.expires_at,
            subject: cert.subject,
            issuer: cert.issuer,
            serial_number: cert.serial_number,
        })
    }

    async fn revoke_certificate(&self, id: Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM certificates WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            anyhow::bail!("Certificate with id {} not found", id);
        }

        Ok(())
    }

    async fn get_certificate(&self, id: Uuid) -> Result<Option<Certificate>> {
        let row = sqlx::query(
            r#"
            SELECT id, certificate_data, created_at, expires_at, subject, issuer, serial_number
            FROM certificates
            WHERE id = ?
            "#
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(Certificate {
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                certificate_data: row.get::<String, _>("certificate_data"),
                created_at: row.get::<chrono::DateTime<Utc>, _>("created_at"),
                expires_at: row.get::<Option<chrono::DateTime<Utc>>, _>("expires_at"),
                subject: row.get::<String, _>("subject"),
                issuer: row.get::<String, _>("issuer"),
                serial_number: row.get::<String, _>("serial_number"),
            }))
        } else {
            Ok(None)
        }
    }

    async fn list_certificates(&self) -> Result<Vec<Certificate>> {
        let rows = sqlx::query(
            r#"
            SELECT id, certificate_data, created_at, expires_at, subject, issuer, serial_number
            FROM certificates
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let certificates = rows
            .into_iter()
            .map(|row| Certificate {
                id: Uuid::parse_str(&row.get::<String, _>("id")).unwrap(),
                certificate_data: row.get::<String, _>("certificate_data"),
                created_at: row.get::<chrono::DateTime<Utc>, _>("created_at"),
                expires_at: row.get::<Option<chrono::DateTime<Utc>>, _>("expires_at"),
                subject: row.get::<String, _>("subject"),
                issuer: row.get::<String, _>("issuer"),
                serial_number: row.get::<String, _>("serial_number"),
            })
            .collect();

        Ok(certificates)
    }

    async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await?;
        Ok(())
    }
}
