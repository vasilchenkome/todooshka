use sqlx::SqlitePool;
use tempfile::TempDir;
use todooshka_app::{RepositoryError, SessionStore, StoredSession, UserStore};
use todooshka_domain::{PasswordHash, Timestamp, User, UserId, Username};
use todooshka_sqlite::{SqliteSessions, SqliteUsers, connect, migrate};

struct Db {
    _dir: TempDir,
    users: SqliteUsers,
    sessions: SqliteSessions,
}

async fn db() -> Db {
    let dir = tempfile::tempdir().expect("временная папка");
    let path = dir.path().join("todooshka.db");
    let url = format!("sqlite://{}", path.to_string_lossy().replace('\\', "/"));
    let pool: SqlitePool = connect(&url).await.expect("база открывается");
    migrate(&pool).await.expect("миграции применяются");

    Db {
        _dir: dir,
        users: SqliteUsers::new(pool.clone()),
        sessions: SqliteSessions::new(pool),
    }
}

fn now(seconds: i64) -> Timestamp {
    Timestamp::from_unix_seconds(seconds)
}

fn username(raw: &str) -> Username {
    Username::parse(raw).expect("логин валиден")
}

fn hash(raw: &str) -> PasswordHash {
    PasswordHash::parse(raw).expect("хеш валиден")
}

async fn add_user(db: &Db, name: &str, password_hash: &str) -> User {
    db.users
        .insert(&username(name), &hash(password_hash), now(100))
        .await
        .expect("пользователь создаётся")
}

fn session(id_hash: &str, user_id: UserId) -> StoredSession {
    StoredSession {
        id_hash: id_hash.to_owned(),
        user_id,
        created_at: now(200),
        expires_at: now(2_000),
        last_seen_at: now(200),
    }
}

#[tokio::test]
async fn insert_and_find_user() {
    let db = db().await;
    let user = add_user(&db, "demo", "argon2-hash").await;

    let found = db
        .users
        .find_by_username(&username("demo"))
        .await
        .expect("поиск")
        .expect("пользователь есть");
    let by_id = db
        .users
        .find_by_id(user.id())
        .await
        .expect("поиск")
        .expect("пользователь есть");

    assert_eq!(found, user);
    assert_eq!(by_id, user);
    assert_eq!(found.username().as_str(), "demo");
    assert_eq!(found.password_hash().as_str(), "argon2-hash");
}

#[tokio::test]
async fn duplicate_username_is_conflict() {
    let db = db().await;
    add_user(&db, "demo", "argon2-hash").await;

    let error = db
        .users
        .insert(&username("demo"), &hash("other-hash"), now(300))
        .await
        .expect_err("дубль логина невозможен");

    assert!(matches!(error, RepositoryError::Conflict(_)));
}

#[tokio::test]
async fn update_password_reports_whether_user_exists() {
    let db = db().await;
    add_user(&db, "demo", "argon2-hash").await;

    assert!(
        db.users
            .update_password(&username("demo"), &hash("new-hash"))
            .await
            .expect("обновление")
    );
    assert!(
        !db.users
            .update_password(&username("nobody"), &hash("new-hash"))
            .await
            .expect("обновление")
    );

    let found = db
        .users
        .find_by_username(&username("demo"))
        .await
        .expect("поиск")
        .expect("пользователь есть");
    assert_eq!(found.password_hash().as_str(), "new-hash");
}

#[tokio::test]
async fn list_users_is_sorted_by_username() {
    let db = db().await;
    add_user(&db, "demo", "hash-a").await;
    add_user(&db, "anton", "hash-b").await;

    let users = db.users.list().await.expect("список");

    assert_eq!(
        users
            .iter()
            .map(|user| user.username().as_str())
            .collect::<Vec<_>>(),
        ["anton", "demo"]
    );
}

#[tokio::test]
async fn session_roundtrip_and_delete() {
    let db = db().await;
    let user = add_user(&db, "demo", "argon2-hash").await;
    let created = session("hash-of-token", user.id());

    db.sessions
        .create(&created)
        .await
        .expect("сессия создаётся");
    let found = db
        .sessions
        .find("hash-of-token")
        .await
        .expect("поиск")
        .expect("сессия есть");
    assert_eq!(found, created);

    db.sessions.delete("hash-of-token").await.expect("удаление");
    assert!(
        db.sessions
            .find("hash-of-token")
            .await
            .expect("поиск")
            .is_none()
    );
}

#[tokio::test]
async fn touch_updates_last_seen_only() {
    let db = db().await;
    let user = add_user(&db, "demo", "argon2-hash").await;
    db.sessions
        .create(&session("hash-of-token", user.id()))
        .await
        .expect("сессия создаётся");

    db.sessions
        .touch("hash-of-token", now(900))
        .await
        .expect("продление");

    let found = db
        .sessions
        .find("hash-of-token")
        .await
        .expect("поиск")
        .expect("сессия есть");
    assert_eq!(found.last_seen_at, now(900));
    assert_eq!(found.expires_at, now(2_000), "срок не сдвигается");
}

#[tokio::test]
async fn delete_for_user_revokes_every_session() {
    let db = db().await;
    let user = add_user(&db, "demo", "argon2-hash").await;
    let other = add_user(&db, "anton", "argon2-hash-2").await;
    db.sessions
        .create(&session("token-one", user.id()))
        .await
        .expect("сессия");
    db.sessions
        .create(&session("token-two", user.id()))
        .await
        .expect("сессия");
    db.sessions
        .create(&session("token-other", other.id()))
        .await
        .expect("сессия");

    db.sessions.delete_for_user(user.id()).await.expect("отзыв");

    assert!(
        db.sessions
            .find("token-one")
            .await
            .expect("поиск")
            .is_none()
    );
    assert!(
        db.sessions
            .find("token-two")
            .await
            .expect("поиск")
            .is_none()
    );
    assert!(
        db.sessions
            .find("token-other")
            .await
            .expect("поиск")
            .is_some()
    );
}

#[tokio::test]
async fn session_of_missing_user_is_rejected_by_foreign_key() {
    let db = db().await;

    let error = db
        .sessions
        .create(&session("orphan", UserId::from_raw(999)))
        .await
        .expect_err("сессия без пользователя невозможна");

    assert!(matches!(error, RepositoryError::Unavailable(_)));
}
