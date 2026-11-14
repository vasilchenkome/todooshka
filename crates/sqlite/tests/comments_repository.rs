use sqlx::SqlitePool;
use tempfile::TempDir;
use todooshka_app::{CommentStore, ProjectRepository, TaskStore};
use todooshka_domain::{ProjectDraft, TaskDraft, TaskId, TaskStatus, Timestamp};
use todooshka_sqlite::{SqliteComments, SqliteProjects, SqliteTasks, connect, migrate};

struct Db {
    _dir: TempDir,
    comments: SqliteComments,
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
        comments: SqliteComments::new(pool.clone()),
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
async fn add_edit_list_and_delete() {
    let db = db().await;
    let (project_id, task_id) = task(&db).await;

    let comment = db
        .comments
        .add(task_id, "Первый", now(200))
        .await
        .expect("добавление")
        .expect("задача есть");
    assert_eq!(comment.body().as_str(), "Первый");

    let edited = db
        .comments
        .edit(comment.id(), "Второй", now(300))
        .await
        .expect("правка")
        .expect("комментарий есть");
    assert_eq!(edited.body().as_str(), "Второй");
    assert_eq!(edited.updated_at(), now(300));

    let listed = db
        .comments
        .list_by_project(project_id)
        .await
        .expect("лента");
    assert_eq!(listed.len(), 1);

    assert!(db.comments.delete(comment.id()).await.expect("удаление"));
    assert!(
        db.comments
            .list_by_project(project_id)
            .await
            .expect("лента")
            .is_empty()
    );
}

#[tokio::test]
async fn missing_task_returns_none() {
    let db = db().await;

    let result = db
        .comments
        .add(TaskId::from_raw(999), "Текст", now(100))
        .await
        .expect("запрос");

    assert!(result.is_none());
}

#[tokio::test]
async fn deleting_task_cascades_comments() {
    let db = db().await;
    let (_, task_id) = task(&db).await;
    db.comments
        .add(task_id, "Текст", now(200))
        .await
        .expect("добавление")
        .expect("задача есть");

    db.tasks.delete(task_id).await.expect("удаление задачи");

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM comments")
        .fetch_one(&db.pool)
        .await
        .expect("счётчик");
    assert_eq!(count, 0);
}
