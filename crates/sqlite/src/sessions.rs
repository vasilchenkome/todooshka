use async_trait::async_trait;
use sqlx::SqlitePool;
use todooshka_app::{RepositoryError, SessionStore, StoredSession};
use todooshka_domain::{Timestamp, UserId};

use crate::mapping::unavailable;

/// SQLite session implementation: `id` is the token hash, the token itself is not stored.
pub struct SqliteSessions {
    pool: SqlitePool,
}

impl SqliteSessions {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct SessionRow {
    id: String,
    user_id: i64,
    created_at: i64,
    expires_at: i64,
    last_seen_at: i64,
}

impl From<SessionRow> for StoredSession {
    fn from(row: SessionRow) -> Self {
        Self {
            id_hash: row.id,
            user_id: UserId::from_raw(row.user_id),
            created_at: Timestamp::from_unix_seconds(row.created_at),
            expires_at: Timestamp::from_unix_seconds(row.expires_at),
            last_seen_at: Timestamp::from_unix_seconds(row.last_seen_at),
        }
    }
}

#[async_trait]
impl SessionStore for SqliteSessions {
    async fn create(&self, session: &StoredSession) -> Result<(), RepositoryError> {
        sqlx::query(
            "INSERT INTO sessions (id, user_id, created_at, expires_at, last_seen_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
        )
        .bind(&session.id_hash)
        .bind(session.user_id.as_raw())
        .bind(session.created_at.as_unix_seconds())
        .bind(session.expires_at.as_unix_seconds())
        .bind(session.last_seen_at.as_unix_seconds())
        .execute(&self.pool)
        .await
        .map_err(unavailable)?;

        Ok(())
    }

    async fn find(&self, id_hash: &str) -> Result<Option<StoredSession>, RepositoryError> {
        let row: Option<SessionRow> = sqlx::query_as(
            "SELECT id, user_id, created_at, expires_at, last_seen_at
             FROM sessions
             WHERE id = ?1",
        )
        .bind(id_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(unavailable)?;

        Ok(row.map(StoredSession::from))
    }

    async fn touch(&self, id_hash: &str, seen_at: Timestamp) -> Result<(), RepositoryError> {
        sqlx::query("UPDATE sessions SET last_seen_at = ?1 WHERE id = ?2")
            .bind(seen_at.as_unix_seconds())
            .bind(id_hash)
            .execute(&self.pool)
            .await
            .map_err(unavailable)?;

        Ok(())
    }

    async fn delete(&self, id_hash: &str) -> Result<(), RepositoryError> {
        sqlx::query("DELETE FROM sessions WHERE id = ?1")
            .bind(id_hash)
            .execute(&self.pool)
            .await
            .map_err(unavailable)?;

        Ok(())
    }

    async fn delete_for_user(&self, user_id: UserId) -> Result<(), RepositoryError> {
        sqlx::query("DELETE FROM sessions WHERE user_id = ?1")
            .bind(user_id.as_raw())
            .execute(&self.pool)
            .await
            .map_err(unavailable)?;

        Ok(())
    }
}
