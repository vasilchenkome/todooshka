use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use todooshka_app::{
    ApplicationError, AuthService, ChangePassword, Clock, CreateUser, Login, PasswordHasher,
    RepositoryError, SESSION_TOUCH_SECONDS, SESSION_TTL_SECONDS, SessionStore, SessionTokens,
    StoredSession, UserStore,
};
use todooshka_domain::{DomainError, PasswordHash, Timestamp, User, UserId, Username};

fn now(seconds: i64) -> Timestamp {
    Timestamp::from_unix_seconds(seconds)
}

struct TestClock(AtomicI64);

impl TestClock {
    fn set(&self, seconds: i64) {
        self.0.store(seconds, Ordering::SeqCst);
    }
}

impl Clock for TestClock {
    fn now(&self) -> Timestamp {
        Timestamp::from_unix_seconds(self.0.load(Ordering::SeqCst))
    }
}

#[derive(Default)]
struct FakeUsers {
    items: Mutex<Vec<User>>,
    next_id: AtomicI64,
}

#[async_trait]
impl UserStore for FakeUsers {
    async fn find_by_username(&self, username: &Username) -> Result<Option<User>, RepositoryError> {
        Ok(self
            .items
            .lock()
            .expect("мьютекс не отравлен")
            .iter()
            .find(|user| user.username() == username)
            .cloned())
    }

    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, RepositoryError> {
        Ok(self
            .items
            .lock()
            .expect("мьютекс не отравлен")
            .iter()
            .find(|user| user.id() == id)
            .cloned())
    }

    async fn insert(
        &self,
        username: &Username,
        password_hash: &PasswordHash,
        now: Timestamp,
    ) -> Result<User, RepositoryError> {
        let mut items = self.items.lock().expect("мьютекс не отравлен");
        if items.iter().any(|user| user.username() == username) {
            return Err(RepositoryError::Conflict("логин занят".to_owned()));
        }

        let id = UserId::from_raw(self.next_id.fetch_add(1, Ordering::SeqCst) + 1);
        let user = User::new(id, username.clone(), password_hash.clone(), now);
        items.push(user.clone());
        Ok(user)
    }

    async fn update_password(
        &self,
        username: &Username,
        password_hash: &PasswordHash,
    ) -> Result<bool, RepositoryError> {
        let mut items = self.items.lock().expect("мьютекс не отравлен");
        let Some(slot) = items.iter_mut().find(|user| user.username() == username) else {
            return Ok(false);
        };
        *slot = User::new(
            slot.id(),
            slot.username().clone(),
            password_hash.clone(),
            slot.created_at(),
        );
        Ok(true)
    }

    async fn list(&self) -> Result<Vec<User>, RepositoryError> {
        Ok(self.items.lock().expect("мьютекс не отравлен").clone())
    }
}

#[derive(Default)]
struct FakeSessions {
    items: Mutex<Vec<StoredSession>>,
}

impl FakeSessions {
    fn stored(&self) -> Vec<StoredSession> {
        self.items.lock().expect("мьютекс не отравлен").clone()
    }
}

#[async_trait]
impl SessionStore for FakeSessions {
    async fn create(&self, session: &StoredSession) -> Result<(), RepositoryError> {
        self.items
            .lock()
            .expect("мьютекс не отравлен")
            .push(session.clone());
        Ok(())
    }

    async fn find(&self, id_hash: &str) -> Result<Option<StoredSession>, RepositoryError> {
        Ok(self
            .items
            .lock()
            .expect("мьютекс не отравлен")
            .iter()
            .find(|session| session.id_hash == id_hash)
            .cloned())
    }

    async fn touch(&self, id_hash: &str, seen_at: Timestamp) -> Result<(), RepositoryError> {
        let mut items = self.items.lock().expect("мьютекс не отравлен");
        if let Some(session) = items.iter_mut().find(|session| session.id_hash == id_hash) {
            session.last_seen_at = seen_at;
        }
        Ok(())
    }

    async fn delete(&self, id_hash: &str) -> Result<(), RepositoryError> {
        self.items
            .lock()
            .expect("мьютекс не отравлен")
            .retain(|session| session.id_hash != id_hash);
        Ok(())
    }

    async fn delete_for_user(&self, user_id: UserId) -> Result<(), RepositoryError> {
        self.items
            .lock()
            .expect("мьютекс не отравлен")
            .retain(|session| session.user_id != user_id);
        Ok(())
    }
}

struct FakeHasher;

impl PasswordHasher for FakeHasher {
    fn hash(&self, password: &str) -> Result<PasswordHash, RepositoryError> {
        Ok(PasswordHash::parse(&format!("fake:{password}")).expect("хеш непустой"))
    }

    fn verify(&self, password: &str, hash: &PasswordHash) -> bool {
        hash.as_str() == format!("fake:{password}")
    }
}

/// Токены детерминированы: проверяем, что в БД уходит только хеш.
struct FakeTokens(AtomicI64);

impl SessionTokens for FakeTokens {
    fn issue(&self) -> (String, String) {
        let number = self.0.fetch_add(1, Ordering::SeqCst) + 1;
        let token = format!("token-{number}");
        let hash = self.hash(&token);
        (token, hash)
    }

    fn hash(&self, token: &str) -> String {
        format!("hash-of-{token}")
    }
}

struct Harness {
    users: Arc<FakeUsers>,
    sessions: Arc<FakeSessions>,
    clock: Arc<TestClock>,
    service: AuthService,
}

fn harness() -> Harness {
    let users = Arc::new(FakeUsers::default());
    let sessions = Arc::new(FakeSessions::default());
    let clock = Arc::new(TestClock(AtomicI64::new(1_000)));

    let service = AuthService::new(
        Arc::clone(&users) as Arc<dyn UserStore>,
        Arc::clone(&sessions) as Arc<dyn SessionStore>,
        Arc::new(FakeHasher) as Arc<dyn PasswordHasher>,
        Arc::new(FakeTokens(AtomicI64::new(0))) as Arc<dyn SessionTokens>,
        Arc::clone(&clock) as Arc<dyn Clock>,
    );

    Harness {
        users,
        sessions,
        clock,
        service,
    }
}

const PASSWORD: &str = "correct-horse-battery";

async fn create_user(h: &Harness, username: &str) -> User {
    h.service
        .create_user(CreateUser {
            username: username.to_owned(),
            password: PASSWORD.to_owned(),
        })
        .await
        .expect("пользователь создаётся")
}

async fn login(h: &Harness, username: &str, password: &str) -> Result<String, ApplicationError> {
    let issued = h
        .service
        .login(Login {
            username: username.to_owned(),
            password: password.to_owned(),
        })
        .await?;
    Ok(issued.token)
}

#[tokio::test]
async fn create_user_stores_only_a_hash() {
    let h = harness();

    let user = create_user(&h, "kirill").await;

    assert_eq!(user.password_hash().as_str(), format!("fake:{PASSWORD}"));
    assert_ne!(user.password_hash().as_str(), PASSWORD);
    assert_eq!(h.users.list().await.expect("список").len(), 1);
}

#[tokio::test]
async fn create_user_rejects_weak_password_before_storage() {
    let h = harness();

    let error = h
        .service
        .create_user(CreateUser {
            username: "kirill".to_owned(),
            password: "short".to_owned(),
        })
        .await
        .expect_err("короткий пароль недопустим");

    assert!(matches!(
        error,
        ApplicationError::Domain(DomainError::WeakPassword { .. })
    ));
    assert!(h.users.list().await.expect("список").is_empty());
}

#[tokio::test]
async fn create_user_rejects_taken_username() {
    let h = harness();
    create_user(&h, "kirill").await;

    let error = h
        .service
        .create_user(CreateUser {
            username: "kirill".to_owned(),
            password: PASSWORD.to_owned(),
        })
        .await
        .expect_err("повторный логин недопустим");

    assert!(matches!(error, ApplicationError::UsernameTaken(_)));
}

#[tokio::test]
async fn login_issues_session_with_hashed_token() {
    let h = harness();
    let user = create_user(&h, "kirill").await;

    let issued = h
        .service
        .login(Login {
            username: "kirill".to_owned(),
            password: PASSWORD.to_owned(),
        })
        .await
        .expect("вход проходит");

    let sessions = h.sessions.stored();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].user_id, user.id());
    assert_eq!(sessions[0].id_hash, format!("hash-of-{}", issued.token));
    assert_ne!(
        sessions[0].id_hash, issued.token,
        "в БД не должно быть токена"
    );
    assert_eq!(
        issued.expires_at,
        now(1_000 + SESSION_TTL_SECONDS),
        "срок сессии — 30 дней"
    );
}

#[tokio::test]
async fn login_does_not_reveal_which_credential_is_wrong() {
    let h = harness();
    create_user(&h, "kirill").await;

    let wrong_password = login(&h, "kirill", "another-password").await;
    let unknown_user = login(&h, "nobody", PASSWORD).await;

    assert_eq!(
        wrong_password.expect_err("неверный пароль"),
        ApplicationError::InvalidCredentials
    );
    assert_eq!(
        unknown_user.expect_err("неизвестный логин"),
        ApplicationError::InvalidCredentials
    );
}

#[tokio::test]
async fn authenticate_returns_user_for_valid_token() {
    let h = harness();
    let user = create_user(&h, "kirill").await;
    let token = login(&h, "kirill", PASSWORD).await.expect("вход");

    let authenticated = h
        .service
        .authenticate(&token)
        .await
        .expect("проверка токена")
        .expect("сессия валидна");

    assert_eq!(authenticated.id, user.id());
    assert_eq!(authenticated.username, "kirill");
}

#[tokio::test]
async fn expired_session_is_dropped() {
    let h = harness();
    create_user(&h, "kirill").await;
    let token = login(&h, "kirill", PASSWORD).await.expect("вход");

    h.clock.set(1_000 + SESSION_TTL_SECONDS + 1);

    assert!(
        h.service
            .authenticate(&token)
            .await
            .expect("проверка")
            .is_none()
    );
    assert!(
        h.sessions.stored().is_empty(),
        "просроченная сессия удаляется"
    );
}

#[tokio::test]
async fn stale_session_activity_is_touched_after_an_hour() {
    let h = harness();
    create_user(&h, "kirill").await;
    let token = login(&h, "kirill", PASSWORD).await.expect("вход");

    h.clock.set(1_000 + SESSION_TOUCH_SECONDS);
    h.service.authenticate(&token).await.expect("проверка");

    assert_eq!(
        h.sessions.stored()[0].last_seen_at,
        now(1_000 + SESSION_TOUCH_SECONDS)
    );
}

#[tokio::test]
async fn fresh_session_is_not_touched() {
    let h = harness();
    create_user(&h, "kirill").await;
    let token = login(&h, "kirill", PASSWORD).await.expect("вход");

    h.clock.set(1_030);
    h.service.authenticate(&token).await.expect("проверка");

    assert_eq!(h.sessions.stored()[0].last_seen_at, now(1_000));
}

#[tokio::test]
async fn logout_removes_session_and_is_idempotent() {
    let h = harness();
    create_user(&h, "kirill").await;
    let token = login(&h, "kirill", PASSWORD).await.expect("вход");

    h.service.logout(&token).await.expect("выход");
    assert!(h.sessions.stored().is_empty());
    assert!(
        h.service
            .authenticate(&token)
            .await
            .expect("проверка")
            .is_none()
    );

    h.service.logout(&token).await.expect("повторный выход");
}

#[tokio::test]
async fn password_change_revokes_sessions_and_old_password_stops_working() {
    let h = harness();
    create_user(&h, "kirill").await;
    let token = login(&h, "kirill", PASSWORD).await.expect("вход");

    h.service
        .change_password(ChangePassword {
            username: "kirill".to_owned(),
            password: "brand-new-password".to_owned(),
        })
        .await
        .expect("пароль меняется");

    assert!(
        h.sessions.stored().is_empty(),
        "смена пароля разлогинивает все устройства"
    );
    assert!(matches!(
        login(&h, "kirill", PASSWORD).await,
        Err(ApplicationError::InvalidCredentials)
    ));
    assert!(login(&h, "kirill", "brand-new-password").await.is_ok());
    assert!(
        h.service
            .authenticate(&token)
            .await
            .expect("проверка")
            .is_none()
    );
}

#[tokio::test]
async fn password_change_for_unknown_user_reports_not_found() {
    let h = harness();

    let error = h
        .service
        .change_password(ChangePassword {
            username: "nobody".to_owned(),
            password: "brand-new-password".to_owned(),
        })
        .await
        .expect_err("пользователя нет");

    assert!(matches!(error, ApplicationError::UserNotFound(_)));
}
