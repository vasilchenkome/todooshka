use sqlx::SqlitePool;
use tempfile::TempDir;
use todooshka_app::{AttachmentStore, ProjectRepository, TaskStore};
use todooshka_domain::{ProjectDraft, TaskDraft, TaskId, TaskStatus, Timestamp};
use todooshka_sqlite::{SqliteAttachments, SqliteProjects, SqliteTasks, connect, migrate};

struct Db {
    _dir: TempDir,
    attachments: SqliteAttachments,
    tasks: SqliteTasks,
    projects: SqliteProjects,
    pool: SqlitePool,
}

async fn db() -> Db {
    let dir = tempfile::tempdir().expect("временная папка");
    let path = dir.path().join("todooshka.db");
    let url = format!("sqlite://{}", path.to_string_lossy().replace('\\', "/"));
    let pool = connect(&url).await.expect("база открывается");
    migrate(&pool).await.expect("миграции применяются");

    Db {
        _dir: dir,
        attachments: SqliteAttachments::new(pool.clone()),
        tasks: SqliteTasks::new(pool.clone()),
        projects: SqliteProjects::new(pool.clone()),
        pool,
    }
}

fn now(seconds: i64) -> Timestamp {
    Timestamp::from_unix_seconds(seconds)
}

async fn task(db: &Db) -> (todooshka_domain::ProjectId, TaskId) {
    let project = db
        .projects
        .create(
            ProjectDraft::new("Проект", "", None).expect("проект"),
            now(100),
        )
        .await
        .expect("проект создан");
    let mut transaction = db.tasks.transaction().await.expect("транзакция");
    let task = transaction
        .create(
            project.id(),
            TaskDraft::new("Задача", "").expect("задача"),
            TaskStatus::Backlog,
            now(100),
        )
        .await
        .expect("задача создана");
    transaction.commit().await.expect("коммит");
    (project.id(), task.id())
}

#[tokio::test]
async fn add_get_list_and_delete() {
    let db = db().await;
    let (project_id, task_id) = task(&db).await;
    let data = b"hello".to_vec();

    let meta = db
        .attachments
        .add(
            task_id,
            "план.txt",
            "text/plain; charset=utf-8",
            &data,
            now(200),
        )
        .await
        .expect("добавление")
        .expect("задача есть");

    assert_eq!(meta.filename.as_str(), "план.txt");
    assert_eq!(meta.size, 5);
    assert_eq!(
        db.attachments
            .count_for_task(task_id)
            .await
            .expect("счётчик"),
        1
    );

    let listed = db
        .attachments
        .list_by_project(project_id)
        .await
        .expect("список");
    assert_eq!(listed.len(), 1);

    let (found_meta, found_data) = db
        .attachments
        .get(meta.id)
        .await
        .expect("чтение")
        .expect("файл есть");
    assert_eq!(found_meta.filename.as_str(), "план.txt");
    assert_eq!(found_data, data);

    assert!(db.attachments.delete(meta.id).await.expect("удаление"));
    assert!(db.attachments.get(meta.id).await.expect("чтение").is_none());
}

#[tokio::test]
async fn adding_to_missing_task_returns_none() {
    let db = db().await;

    let result = db
        .attachments
        .add(TaskId::from_raw(999), "a.txt", "text/plain", b"x", now(100))
        .await
        .expect("запрос");

    assert!(result.is_none());
}

#[tokio::test]
async fn deleting_task_cascades_attachments() {
    let db = db().await;
    let (_, task_id) = task(&db).await;
    db.attachments
        .add(task_id, "a.txt", "text/plain", b"x", now(100))
        .await
        .expect("добавление")
        .expect("задача есть");

    db.tasks.delete(task_id).await.expect("удаление задачи");

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM attachments")
        .fetch_one(&db.pool)
        .await
        .expect("счётчик");
    assert_eq!(count, 0);
}
