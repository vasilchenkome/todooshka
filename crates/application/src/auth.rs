use std::sync::Arc;

use todooshka_domain::{Timestamp, User, UserId, Username, validate_password};

use crate::error::ApplicationError;
use crate::ports::{Clock, PasswordHasher, SessionStore, SessionTokens, StoredSession, UserStore};

/// A session lives 30 days; activity refreshes the timestamp, not the expiry.
pub const SESSION_TTL_SECONDS: i64 = 30 * 24 * 60 * 60;

/// We refresh last-activity at most once an hour to avoid a write on every request.
pub const SESSION_TOUCH_SECONDS: i64 = 60 * 60;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Login {
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreateUser {
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChangePassword {
    pub username: String,
    pub password: String,
}

/// An issued session: the token goes into a cookie, only its hash is stored in the DB.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IssuedSession {
    pub token: String,
    pub expires_at: Timestamp,
    pub user: User,
}

/// Who arrived with a valid session: enough for the middleware and `/api/auth/me`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthenticatedUser {
    pub id: UserId,
    pub username: String,
}

/// Sign-in and account use cases.
pub struct AuthService {
    users: Arc<dyn UserStore>,
    sessions: Arc<dyn SessionStore>,
    hasher: Arc<dyn PasswordHasher>,
    tokens: Arc<dyn SessionTokens>,
    clock: Arc<dyn Clock>,
}

impl AuthService {
    pub fn new(
        users: Arc<dyn UserStore>,
        sessions: Arc<dyn SessionStore>,
        hasher: Arc<dyn PasswordHasher>,
        tokens: Arc<dyn SessionTokens>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            users,
            sessions,
            hasher,
            tokens,
            clock,
        }
    }

    pub async fn create_user(&self, command: CreateUser) -> Result<User, ApplicationError> {
        let username = Username::parse(&command.username)?;
        validate_password(&command.password)?;
        let password_hash = self.hasher.hash(&command.password)?;

        match self
            .users
            .insert(&username, &password_hash, self.clock.now())
            .await
        {
            Ok(user) => Ok(user),
            // The unique index in the DB is the only reliable check that a login is taken.
            Err(crate::RepositoryError::Conflict(_)) => {
                Err(ApplicationError::UsernameTaken(username))
            }
            Err(error) => Err(error.into()),
        }
    }

    pub async fn change_password(&self, command: ChangePassword) -> Result<(), ApplicationError> {
        let username = Username::parse(&command.username)?;
        validate_password(&command.password)?;

        let user = self
            .users
            .find_by_username(&username)
            .await?
            .ok_or_else(|| ApplicationError::UserNotFound(username.clone()))?;

        let password_hash = self.hasher.hash(&command.password)?;
        if !self
            .users
            .update_password(&username, &password_hash)
            .await?
        {
            return Err(ApplicationError::UserNotFound(username));
        }

        // Changing the password signs out every device: the old tokens stop working.
        self.sessions.delete_for_user(user.id()).await?;
        Ok(())
    }

    pub async fn list_users(&self) -> Result<Vec<User>, ApplicationError> {
        Ok(self.users.list().await?)
    }

    /// The same error for an unknown login and a wrong password: we do not reveal
    /// which accounts exist.
    pub async fn login(&self, command: Login) -> Result<IssuedSession, ApplicationError> {
        let username =
            Username::parse(&command.username).map_err(|_| ApplicationError::InvalidCredentials)?;
        let user = self
            .users
            .find_by_username(&username)
            .await?
            .ok_or(ApplicationError::InvalidCredentials)?;

        if !self.hasher.verify(&command.password, user.password_hash()) {
            return Err(ApplicationError::InvalidCredentials);
        }

        let now = self.clock.now();
        let (token, id_hash) = self.tokens.issue();
        let session = StoredSession {
            id_hash,
            user_id: user.id(),
            created_at: now,
            expires_at: Timestamp::from_unix_seconds(now.as_unix_seconds() + SESSION_TTL_SECONDS),
            last_seen_at: now,
        };
        self.sessions.create(&session).await?;

        Ok(IssuedSession {
            token,
            expires_at: session.expires_at,
            user,
        })
    }

    /// Validates the cookie token. Drops an expired session; refreshes activity at most hourly.
    pub async fn authenticate(
        &self,
        token: &str,
    ) -> Result<Option<AuthenticatedUser>, ApplicationError> {
        if token.is_empty() {
            return Ok(None);
        }

        let id_hash = self.tokens.hash(token);
        let Some(session) = self.sessions.find(&id_hash).await? else {
            return Ok(None);
        };

        let now = self.clock.now();
        if session.expires_at <= now {
            self.sessions.delete(&id_hash).await?;
            return Ok(None);
        }

        if now.as_unix_seconds() - session.last_seen_at.as_unix_seconds() >= SESSION_TOUCH_SECONDS {
            self.sessions.touch(&id_hash, now).await?;
        }

        let Some(user) = self.users.find_by_id(session.user_id).await? else {
            self.sessions.delete(&id_hash).await?;
            return Ok(None);
        };

        Ok(Some(AuthenticatedUser {
            id: user.id(),
            username: user.username().as_str().to_owned(),
        }))
    }

    pub async fn logout(&self, token: &str) -> Result<(), ApplicationError> {
        if token.is_empty() {
            return Ok(());
        }
        self.sessions.delete(&self.tokens.hash(token)).await?;
        Ok(())
    }
}
