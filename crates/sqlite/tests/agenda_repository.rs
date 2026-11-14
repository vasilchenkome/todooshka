use tempfile::TempDir;
use todooshka_app::{AgendaReader, ProjectRepository, TaskStore};
use todooshka_domain::{
    DueDate, Planning, Priority, ProjectDraft, ProjectId, TaskId, TaskStatus, Timestamp,
};
use todooshka_sqlite::{SqliteAgenda, SqliteProjects, SqliteTasks, connect, migrate};

struct Db {
    _dir: TempDir,
    projects: SqliteProjects,
    tasks: SqliteTasks,
    agenda: SqliteAgenda,
}

async fn db() -> Db {
    let dir = tempfile::tempdir().expect("временная папка");
    let path = dir.path().join("todooshka.db");
    let url = format!("sqlite://{}", path.to_string_lossy().replace('\\', "/"));
    let pool = connect(&url).await.expect("база открывается");
    migrate(&pool).await.expect("миграции применяются");

    Db {
        _dir: dir,
        projects: SqliteProjects::new(pool.clone()),
        tasks: SqliteTasks::new(pool.clone()),
        agenda: SqliteAgenda::new(pool.clone()),
    }
}

fn now(seconds: i64) -> Timestamp {
    Timestamp::from_unix_seconds(seconds)
}

async fn project(db: &Db, name: &str) -> ProjectId {
    db.projects
        .create(
            ProjectDraft::new(name, "", None).expect("черновик"),
            now(100),
        )
        .await
        .expect("проект создаётся")
        .id()
}

async fn task(db: &Db, project_id: ProjectId, title: &str, status: TaskStatus) -> TaskId {
    let mut tx = db.tasks.transaction().await.expect("транзакция");
    let task = tx
        .create(
            project_id,
            todooshka_domain::TaskDraft::new(title, "").expect("черновик"),
            status,
            now(100),
        )
        .await
        .expect("задача создаётся");
    tx.commit().await.expect("коммит");
    task.id()
}

async fn plan(db: &Db, id: TaskId, due: Option<&str>) {
    let mut tx = db.tasks.transaction().await.expect("транзакция");
    let mut task = tx.get(id).await.expect("чтение").expect("задача есть");
    let due = due.map(|value| DueDate::parse(value).expect("дата"));
    task.plan(Planning::new(due, Priority::High, None), now(200));
    tx.save(&task).await.expect("запись");
    tx.commit().await.expect("коммит");
}

#[tokio::test]
async fn agenda_reads_open_tasks_across_projects_and_ignores_archive_and_done() {
    let db = db().await;
    let active = project(&db, "Активный").await;
    let archived = project(&db, "Архивный").await;

    let overdue = task(&db, active, "Просрочено", TaskStatus::Backlog).await;
    plan(&db, overdue, Some("2026-11-12")).await;
    let unplanned = task(&db, active, "Без срока", TaskStatus::Backlog).await;
    let done = task(&db, active, "Готово", TaskStatus::Done).await;
    plan(&db, done, Some("2026-11-13")).await;
    let archived_task = task(&db, archived, "Из архива", TaskStatus::Backlog).await;
    plan(&db, archived_task, Some("2026-11-14")).await;

    let mut project = db
        .projects
        .get(archived)
        .await
        .expect("чтение")
        .expect("проект");
    project.archive(now(300));
    db.projects.save(&project).await.expect("архив");

    let entries = db
        .agenda
        .list_open_due_until(&DueDate::parse("2026-11-21").expect("дата"))
        .await
        .expect("agenda");

    let titles: Vec<&str> = entries
        .iter()
        .map(|entry| entry.task.title().as_str())
        .collect();
    assert_eq!(titles.len(), 2);
    assert!(titles.contains(&"Просрочено"));
    assert!(titles.contains(&"Из архива"));
    assert!(!titles.contains(&"Без срока"));
    assert!(!titles.contains(&"Готово"));
    assert!(
        entries
            .iter()
            .find(|entry| entry.task.id() == archived_task)
            .is_some_and(|entry| entry.project_name == "Архивный")
    );
    assert!(!entries.iter().any(|entry| entry.task.id() == unplanned));
}
