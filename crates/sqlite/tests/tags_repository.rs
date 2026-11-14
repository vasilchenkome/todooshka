use sqlx::SqlitePool;
use tempfile::TempDir;
use todooshka_app::{ProjectRepository, TagStore, TaskStore};
use todooshka_domain::{ProjectDraft, TaskDraft, TaskId, TaskStatus, Timestamp};
use todooshka_sqlite::{SqliteProjects, SqliteTags, SqliteTasks, connect, migrate};

struct Db {
    _dir: TempDir,
    tags: SqliteTags,
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
        tags: SqliteTags::new(pool.clone()),
        tasks: SqliteTasks::new(pool.clone()),
        projects: SqliteProjects::new(pool.clone()),
        pool,
    }
}

fn now(seconds: i64) -> Timestamp {
    Timestamp::from_unix_seconds(seconds)
}

async fn task(db: &Db) -> todooshka_domain::ProjectId {
    let project = db
        .projects
        .create(
            ProjectDraft::new("Проект", "", None).expect("проект"),
            now(100),
        )
        .await
        .expect("проект создан");
    let mut transaction = db.tasks.transaction().await.expect("транзакция");
    transaction
        .create(
            project.id(),
            TaskDraft::new("Задача", "").expect("задача"),
            TaskStatus::Backlog,
            now(100),
        )
        .await
        .expect("задача создана");
    transaction.commit().await.expect("коммит");
    project.id()
}

#[tokio::test]
async fn replace_links_tags_to_task_and_project() {
    let db = db().await;
    let project_id = task(&db).await;

    let mut transaction = db.tasks.transaction().await.expect("транзакция");
    let task = transaction
        .list_column(project_id, TaskStatus::Backlog)
        .await
        .expect("колонка")
        .remove(0);
    transaction.commit().await.expect("коммит");

    let tags = db
        .tags
        .replace(
            task.id(),
            &[
                todooshka_domain::TagName::parse("банк").expect("тег"),
                todooshka_domain::TagName::parse("релиз").expect("тег"),
            ],
        )
        .await
        .expect("замена")
        .expect("задача есть");

    assert_eq!(tags.len(), 2);

    let links = db.tags.list_by_project(project_id).await.expect("связи");
    assert_eq!(links.len(), 2);
    assert_eq!(links[0].task_id, task.id());

    let all = db.tags.list_all().await.expect("список тегов");
    assert_eq!(all.len(), 2);

    let empty = db
        .tags
        .replace(task.id(), &[])
        .await
        .expect("очистка")
        .expect("задача есть");
    assert!(empty.is_empty());
    assert!(
        db.tags
            .list_by_project(project_id)
            .await
            .expect("связи")
            .is_empty()
    );
}

#[tokio::test]
async fn replace_for_missing_task_returns_none() {
    let db = db().await;

    let result = db
        .tags
        .replace(TaskId::from_raw(999), &[])
        .await
        .expect("запрос");

    assert!(result.is_none());
}

#[tokio::test]
async fn deleting_task_removes_links_but_keeps_tags() {
    let db = db().await;
    let project_id = task(&db).await;
    let mut transaction = db.tasks.transaction().await.expect("транзакция");
    let task = transaction
        .list_column(project_id, TaskStatus::Backlog)
        .await
        .expect("колонка")
        .remove(0);
    transaction.commit().await.expect("коммит");
    db.tags
        .replace(
            task.id(),
            &[todooshka_domain::TagName::parse("банк").expect("тег")],
        )
        .await
        .expect("замена");

    db.tasks.delete(task.id()).await.expect("удаление задачи");

    let links: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM task_tags")
        .fetch_one(&db.pool)
        .await
        .expect("счётчик");
    let tags: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tags")
        .fetch_one(&db.pool)
        .await
        .expect("счётчик");
    assert_eq!(links, 0);
    assert_eq!(tags, 1);
}
