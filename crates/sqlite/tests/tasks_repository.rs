use sqlx::SqlitePool;
use tempfile::TempDir;
use todooshka_app::{ProjectRepository, RepositoryError, TaskStore};
use todooshka_domain::{
    DueDate, EstimateMinutes, Planning, Priority, ProjectDraft, ProjectId, Task, TaskDraft, TaskId,
    TaskStatus, Timestamp,
};
use todooshka_sqlite::{SqliteProjects, SqliteTasks, connect, migrate};

struct Db {
    _dir: TempDir,
    pool: SqlitePool,
    projects: SqliteProjects,
    tasks: SqliteTasks,
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
        pool,
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

async fn add(db: &Db, project_id: ProjectId, title: &str, status: TaskStatus) -> Task {
    let mut transaction = db.tasks.transaction().await.expect("транзакция");
    let task = transaction
        .create(
            project_id,
            TaskDraft::new(title, "").expect("черновик"),
            status,
            now(100),
        )
        .await
        .expect("задача создаётся");
    transaction.commit().await.expect("коммит");
    task
}

#[tokio::test]
async fn create_appends_positions_within_each_column() {
    let db = db().await;
    let project_id = project(&db, "Проект").await;

    let first = add(&db, project_id, "Первая", TaskStatus::Backlog).await;
    let second = add(&db, project_id, "Вторая", TaskStatus::Backlog).await;
    let done = add(&db, project_id, "Готово", TaskStatus::Done).await;

    assert_eq!(first.position(), 1.0);
    assert_eq!(second.position(), 2.0);
    assert_eq!(done.position(), 1.0, "колонки нумеруются независимо");
    assert_eq!(done.completed_at(), Some(now(100)));
}

#[tokio::test]
async fn list_by_project_returns_every_column() {
    let db = db().await;
    let project_id = project(&db, "Проект").await;
    add(&db, project_id, "Бэклог", TaskStatus::Backlog).await;
    add(&db, project_id, "В работе", TaskStatus::InProgress).await;
    add(&db, project_id, "Готово", TaskStatus::Done).await;

    let tasks = db.tasks.list_by_project(project_id).await.expect("список");

    assert_eq!(tasks.len(), 3);
    assert_eq!(tasks[0].title().as_str(), "Бэклог");
}

#[tokio::test]
async fn committed_move_persists_status_and_completion() {
    let db = db().await;
    let project_id = project(&db, "Проект").await;
    let task = add(&db, project_id, "Задача", TaskStatus::Backlog).await;

    let mut transaction = db.tasks.transaction().await.expect("транзакция");
    let mut stored = transaction
        .get(task.id())
        .await
        .expect("чтение")
        .expect("задача есть");
    stored
        .move_to(TaskStatus::Done, 1.0, now(200))
        .expect("перенос");
    transaction.save(&stored).await.expect("запись");
    transaction.commit().await.expect("коммит");

    let mut check = db.tasks.transaction().await.expect("транзакция");
    let reloaded = check
        .get(task.id())
        .await
        .expect("чтение")
        .expect("задача есть");

    assert_eq!(reloaded.status(), TaskStatus::Done);
    assert_eq!(reloaded.completed_at(), Some(now(200)));
    assert_eq!(reloaded.updated_at(), now(200));
}

#[tokio::test]
async fn dropped_transaction_rolls_back() {
    let db = db().await;
    let project_id = project(&db, "Проект").await;
    let task = add(&db, project_id, "Задача", TaskStatus::Backlog).await;

    {
        let mut transaction = db.tasks.transaction().await.expect("транзакция");
        let mut stored = transaction
            .get(task.id())
            .await
            .expect("чтение")
            .expect("задача есть");
        stored
            .move_to(TaskStatus::Done, 1.0, now(200))
            .expect("перенос");
        transaction.save(&stored).await.expect("запись");
        // No commit: the transaction must roll back when dropped.
    }

    let mut check = db.tasks.transaction().await.expect("транзакция");
    let reloaded = check
        .get(task.id())
        .await
        .expect("чтение")
        .expect("задача есть");

    assert_eq!(reloaded.status(), TaskStatus::Backlog);
    assert_eq!(reloaded.completed_at(), None);
}

#[tokio::test]
async fn save_of_unknown_task_reports_conflict() {
    let db = db().await;
    let project_id = project(&db, "Проект").await;
    let missing = Task::create(
        TaskId::from_raw(999),
        project_id,
        TaskDraft::new("Призрак", "").expect("черновик"),
        TaskStatus::Backlog,
        1.0,
        now(100),
    )
    .expect("задача");

    let mut transaction = db.tasks.transaction().await.expect("транзакция");
    let error = transaction
        .save(&missing)
        .await
        .expect_err("сохранение в пустую таблицу должно падать");

    assert!(matches!(error, RepositoryError::Conflict(_)));
}

#[tokio::test]
async fn delete_reports_whether_task_existed() {
    let db = db().await;
    let project_id = project(&db, "Проект").await;
    let task = add(&db, project_id, "Задача", TaskStatus::Backlog).await;

    assert!(db.tasks.delete(task.id()).await.expect("удаление"));
    assert!(
        !db.tasks
            .delete(task.id())
            .await
            .expect("повторное удаление")
    );
}

#[tokio::test]
async fn database_rejects_unknown_status() {
    let db = db().await;
    let project_id = project(&db, "Проект").await;

    let result = sqlx::query(
        "INSERT INTO tasks (project_id, title, description, status, position,
                            created_at, updated_at, completed_at)
         VALUES (?1, 'Задача', '', 'archived', 1.0, 1, 1, NULL)",
    )
    .bind(project_id.as_raw())
    .execute(&db.pool)
    .await;

    assert!(
        result.is_err(),
        "CHECK-ограничение обязано отсекать неизвестный статус до маппинга"
    );
}

#[tokio::test]
async fn inconsistent_completion_is_reported_as_corruption() {
    let db = db().await;
    let project_id = project(&db, "Проект").await;
    sqlx::query(
        "INSERT INTO tasks (project_id, title, description, status, position,
                            created_at, updated_at, completed_at)
         VALUES (?1, 'Задача', '', 'done', 1.0, 1, 1, NULL)",
    )
    .bind(project_id.as_raw())
    .execute(&db.pool)
    .await
    .expect("строка вставлена в обход репозитория");

    let mut transaction = db.tasks.transaction().await.expect("транзакция");
    let error = transaction
        .get(TaskId::from_raw(1))
        .await
        .expect_err("done без completed_at — порча данных");

    assert!(matches!(error, RepositoryError::Corrupted(_)));
}

#[tokio::test]
async fn counts_by_project_groups_tasks_by_status() {
    let db = db().await;
    let project_id = project(&db, "Проект").await;
    let other = project(&db, "Другой").await;
    add(&db, project_id, "Бэклог", TaskStatus::Backlog).await;
    add(&db, project_id, "В работе", TaskStatus::InProgress).await;
    add(&db, project_id, "Готово", TaskStatus::Done).await;
    add(&db, other, "Чужой бэклог", TaskStatus::Backlog).await;

    let mut counts = db.tasks.counts_by_project().await.expect("счётчики");

    counts.sort_by_key(|count| (count.project_id.as_raw(), count.status.order()));
    let mine: Vec<(TaskStatus, i64)> = counts
        .iter()
        .filter(|count| count.project_id == project_id)
        .map(|count| (count.status, count.tasks))
        .collect();

    assert_eq!(
        mine,
        vec![
            (TaskStatus::Backlog, 1),
            (TaskStatus::InProgress, 1),
            (TaskStatus::Done, 1)
        ]
    );
    assert_eq!(
        counts
            .iter()
            .filter(|count| count.project_id == other)
            .map(|count| count.tasks)
            .sum::<i64>(),
        1
    );
}

#[tokio::test]
async fn list_column_isolates_project_and_status() {
    let db = db().await;
    let first = project(&db, "Первый").await;
    let second = project(&db, "Второй").await;
    add(&db, first, "Бэклог первого", TaskStatus::Backlog).await;
    add(&db, first, "В работе первого", TaskStatus::InProgress).await;
    add(&db, second, "Бэклог второго", TaskStatus::Backlog).await;

    let mut transaction = db.tasks.transaction().await.expect("транзакция");
    let column = transaction
        .list_column(first, TaskStatus::Backlog)
        .await
        .expect("колонка");

    assert_eq!(column.len(), 1);
    assert_eq!(column[0].title().as_str(), "Бэклог первого");
}

#[tokio::test]
async fn planning_roundtrips_through_sqlite() {
    let db = db().await;
    let project_id = project(&db, "Проект").await;
    let task = add(&db, project_id, "Задача", TaskStatus::Backlog).await;

    let planning = Planning::new(
        Some(DueDate::parse("2026-11-20").expect("дата")),
        Priority::Urgent,
        Some(EstimateMinutes::parse(90).expect("оценка")),
    );

    let mut transaction = db.tasks.transaction().await.expect("транзакция");
    let mut stored = transaction
        .get(task.id())
        .await
        .expect("чтение")
        .expect("задача есть");
    stored.plan(planning.clone(), now(200));
    transaction.save(&stored).await.expect("запись");
    transaction.commit().await.expect("коммит");

    let mut check = db.tasks.transaction().await.expect("транзакция");
    let reloaded = check
        .get(task.id())
        .await
        .expect("чтение")
        .expect("задача есть");

    assert_eq!(reloaded.planning(), &planning);
    assert_eq!(reloaded.updated_at(), now(200));
}

#[tokio::test]
async fn new_task_has_default_planning() {
    let db = db().await;
    let project_id = project(&db, "Проект").await;
    let task = add(&db, project_id, "Задача", TaskStatus::Backlog).await;

    assert_eq!(task.planning(), &Planning::default());
    assert_eq!(task.planning().priority(), Priority::Normal);
    assert!(task.planning().due_on().is_none());
    assert!(task.planning().estimate().is_none());
}
