use async_trait::async_trait;
use sqlx::SqlitePool;
use todooshka_app::{RepositoryError, UserStore};
use todooshka_domain::{PasswordHash, Timestamp, User, UserId, Username};

use crate::mapping::{corrupted, unavailable};

/// SQLite account implementation. Passwords arrive already hashed.
pub struct SqliteUsers {
    pool: SqlitePool,
}

impl SqliteUsers {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct UserRow {
    id: i64,
    username: String,
    password_hash: String,
    created_at: i64,
}

impl UserRow {
    fn into_domain(self) -> Result<User, RepositoryError> {
        Ok(User::new(
            UserId::from_raw(self.id),
            Username::parse(&self.username).map_err(corrupted)?,
            PasswordHash::parse(&self.password_hash).map_err(corrupted)?,
            Timestamp::from_unix_seconds(self.created_at),
        ))
    }
}

const SELECT_USER: &str = "SELECT id, username, password_hash, created_at FROM users";

#[async_trait]
impl UserStore for SqliteUsers {
    async fn find_by_username(&self, username: &Username) -> Result<Option<User>, RepositoryError> {
        let row: Option<UserRow> = sqlx::query_as(
            "SELECT id, username, password_hash, created_at
             FROM users
             WHERE username = ?1",
        )
        .bind(username.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(unavailable)?;

        row.map(UserRow::into_domain).transpose()
    }

    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, RepositoryError> {
        let row: Option<UserRow> = sqlx::query_as(
            "SELECT id, username, password_hash, created_at
             FROM users
             WHERE id = ?1",
        )
        .bind(id.as_raw())
        .fetch_optional(&self.pool)
        .await
        .map_err(unavailable)?;

        row.map(UserRow::into_domain).transpose()
    }

    async fn insert(
        &self,
        username: &Username,
        password_hash: &PasswordHash,
        now: Timestamp,
    ) -> Result<User, RepositoryError> {
        let result: Result<UserRow, sqlx::Error> = sqlx::query_as(
            "INSERT INTO users (username, password_hash, created_at)
             VALUES (?1, ?2, ?3)
             RETURNING id, username, password_hash, created_at",
        )
        .bind(username.as_str())
        .bind(password_hash.as_str())
        .bind(now.as_unix_seconds())
        .fetch_one(&self.pool)
        .await;

        match result {
            Ok(row) => row.into_domain(),
            Err(error) if is_unique_violation(&error) => Err(RepositoryError::Conflict(format!(
                "логин {} уже занят",
                username
            ))),
            Err(error) => Err(unavailable(error)),
        }
    }

    async fn update_password(
        &self,
        username: &Username,
        password_hash: &PasswordHash,
    ) -> Result<bool, RepositoryError> {
        let result = sqlx::query("UPDATE users SET password_hash = ?1 WHERE username = ?2")
            .bind(password_hash.as_str())
            .bind(username.as_str())
            .execute(&self.pool)
            .await
            .map_err(unavailable)?;

        Ok(result.rows_affected() > 0)
    }

    async fn list(&self) -> Result<Vec<User>, RepositoryError> {
        let rows: Vec<UserRow> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
            "{SELECT_USER} ORDER BY username ASC"
        )))
        .fetch_all(&self.pool)
        .await
        .map_err(unavailable)?;

        rows.into_iter().map(UserRow::into_domain).collect()
    }
}

fn is_unique_violation(error: &sqlx::Error) -> bool {
    error
        .as_database_error()
        .is_some_and(|database_error| database_error.is_unique_violation())
}
