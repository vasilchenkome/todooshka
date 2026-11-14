use sqlx::SqlitePool;
use tempfile::TempDir;
use todooshka_app::{ProjectRepository, SubtaskStore, TaskStore};
use todooshka_domain::{
    ProjectDraft, ProjectId, SubtaskId, TaskDraft, TaskId, TaskStatus, Timestamp,
};
use todooshka_sqlite::{SqliteProjects, SqliteSubtasks, SqliteTasks, connect, migrate};

struct Db {
    _dir: TempDir,
    subtasks: SqliteSubtasks,
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
        subtasks: SqliteSubtasks::new(pool.clone()),
        tasks: SqliteTasks::new(pool.clone()),
        projects: SqliteProjects::new(pool.clone()),
        pool,
    }
}

fn now(seconds: i64) -> Timestamp {
    Timestamp::from_unix_seconds(seconds)
}

async fn task(db: &Db) -> (ProjectId, TaskId) {
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
async fn add_list_toggle_and_delete() {
    let db = db().await;
    let (project_id, task_id) = task(&db).await;

    let first = db
        .subtasks
        .add(task_id, "Первый", now(200))
        .await
        .expect("добавление")
        .expect("задача есть");
    let second = db
        .subtasks
        .add(task_id, "Второй", now(200))
        .await
        .expect("добавление")
        .expect("задача есть");

    assert_eq!(first.position(), 1.0);
    assert_eq!(second.position(), 2.0);
    assert!(!first.is_done());

    let listed = db
        .subtasks
        .list_by_project(project_id)
        .await
        .expect("список");
    assert_eq!(listed.len(), 2);
    assert_eq!(listed[0].title().as_str(), "Первый");

    let toggled = db
        .subtasks
        .update(first.id(), None, Some(true), now(300))
        .await
        .expect("обновление")
        .expect("пункт есть");
    assert!(toggled.is_done());
    assert_eq!(toggled.updated_at(), now(300));

    assert!(db.subtasks.delete(second.id()).await.expect("удаление"));
    assert!(
        !db.subtasks
            .delete(SubtaskId::from_raw(999))
            .await
            .expect("удаление")
    );
}

#[tokio::test]
async fn adding_to_missing_task_returns_none() {
    let db = db().await;

    let result = db
        .subtasks
        .add(TaskId::from_raw(999), "Пункт", now(100))
        .await
        .expect("запрос выполнен");

    assert!(result.is_none());
}

#[tokio::test]
async fn deleting_task_cascades_subtasks() {
    let db = db().await;
    let (_, task_id) = task(&db).await;
    db.subtasks
        .add(task_id, "Пункт", now(200))
        .await
        .expect("добавление")
        .expect("задача есть");

    db.tasks.delete(task_id).await.expect("удаление задачи");

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM subtasks")
        .fetch_one(&db.pool)
        .await
        .expect("счётчик");
    assert_eq!(count, 0);
}
