use sqlx::SqlitePool;
use tempfile::TempDir;
use todooshka_sqlite::{StorageError, connect, migrate};

async fn open(dir: &TempDir) -> SqlitePool {
    let path = dir.path().join("todooshka.db");
    let url = format!("sqlite://{}", path.to_string_lossy().replace('\\', "/"));
    connect(&url).await.expect("база должна открываться")
}

#[tokio::test]
async fn migrations_create_mvp_schema() {
    let dir = tempfile::tempdir().expect("временная папка");
    let pool = open(&dir).await;

    migrate(&pool).await.expect("миграции применяются");

    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master
         WHERE type = 'table' AND name NOT LIKE 'sqlite_%'
         ORDER BY name",
    )
    .fetch_all(&pool)
    .await
    .expect("схема читается");

    assert_eq!(
        tables,
        vec!["_sqlx_migrations", "projects", "sessions", "tasks", "users"],
        "схема MVP должна совпадать с миграцией 0001"
    );
}

#[tokio::test]
async fn migrate_twice_is_safe() {
    let dir = tempfile::tempdir().expect("временная папка");
    let pool = open(&dir).await;

    migrate(&pool).await.expect("первый запуск");
    migrate(&pool)
        .await
        .expect("повторный запуск не должен падать");

    let applied: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations")
        .fetch_one(&pool)
        .await
        .expect("таблица миграций читается");

    assert_eq!(
        applied, 1,
        "миграция 0001 должна примениться ровно один раз"
    );
}

#[tokio::test]
async fn connection_uses_wal_and_foreign_keys() {
    let dir = tempfile::tempdir().expect("временная папка");
    let pool = open(&dir).await;

    let journal: String = sqlx::query_scalar("PRAGMA journal_mode")
        .fetch_one(&pool)
        .await
        .expect("journal_mode читается");
    let foreign_keys: i64 = sqlx::query_scalar("PRAGMA foreign_keys")
        .fetch_one(&pool)
        .await
        .expect("foreign_keys читается");

    assert_eq!(
        journal.to_lowercase(),
        "wal",
        "нужен WAL для одного файла и читателей"
    );
    assert_eq!(foreign_keys, 1, "внешние ключи должны быть включены");
}

#[tokio::test]
async fn non_sqlite_url_is_rejected() {
    let error = connect("postgres://localhost/todooshka")
        .await
        .expect_err("чужой драйвер должен падать до подключения");

    assert!(matches!(error, StorageError::InvalidUrl(_)));
}
