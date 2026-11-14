use sqlx::SqlitePool;
use tempfile::TempDir;
use todooshka_app::{ProjectRepository, RepositoryError};
use todooshka_domain::{Project, ProjectColor, ProjectDraft, ProjectId, ProjectName, Timestamp};
use todooshka_sqlite::{SqliteProjects, connect, migrate};

struct Db {
    _dir: TempDir,
    pool: SqlitePool,
    repo: SqliteProjects,
}

async fn db() -> Db {
    let dir = tempfile::tempdir().expect("временная папка");
    let path = dir.path().join("todooshka.db");
    let url = format!("sqlite://{}", path.to_string_lossy().replace('\\', "/"));
    let pool = connect(&url).await.expect("база открывается");
    migrate(&pool).await.expect("миграции применяются");
    let repo = SqliteProjects::new(pool.clone());

    Db {
        _dir: dir,
        pool,
        repo,
    }
}

fn draft(name: &str) -> ProjectDraft {
    ProjectDraft::new(name, "", None).expect("черновик валиден")
}

fn now(seconds: i64) -> Timestamp {
    Timestamp::from_unix_seconds(seconds)
}

#[tokio::test]
async fn create_appends_projects_to_the_end() {
    let db = db().await;

    let first = db
        .repo
        .create(draft("Первый"), now(100))
        .await
        .expect("первый создаётся");
    let second = db
        .repo
        .create(draft("Второй"), now(100))
        .await
        .expect("второй создаётся");
    let third = db
        .repo
        .create(draft("Третий"), now(100))
        .await
        .expect("третий создаётся");

    assert_eq!(
        (first.position(), second.position(), third.position()),
        (0, 1, 2)
    );
    assert!(first.id().as_raw() < second.id().as_raw());

    let names: Vec<String> = db
        .repo
        .list(false)
        .await
        .expect("список читается")
        .iter()
        .map(|project| project.name().as_str().to_owned())
        .collect();
    assert_eq!(names, ["Первый", "Второй", "Третий"]);
}

#[tokio::test]
async fn list_hides_archived_projects_by_default() {
    let db = db().await;
    let active = db
        .repo
        .create(draft("Активный"), now(100))
        .await
        .expect("проект создаётся");
    let mut archive = db
        .repo
        .create(draft("В архив"), now(100))
        .await
        .expect("проект создаётся");
    archive.archive(now(200));
    db.repo.save(&archive).await.expect("архив сохраняется");

    let visible = db.repo.list(false).await.expect("активные");
    let all = db.repo.list(true).await.expect("все");

    assert_eq!(
        visible
            .iter()
            .map(|project| project.id())
            .collect::<Vec<_>>(),
        vec![active.id()]
    );
    assert_eq!(all.len(), 2);
}

#[tokio::test]
async fn get_returns_none_for_missing_project() {
    let db = db().await;

    let project = db.repo.get(ProjectId::from_raw(404)).await.expect("запрос");

    assert!(project.is_none());
}

#[tokio::test]
async fn save_persists_every_editable_field() {
    let db = db().await;
    let project = db
        .repo
        .create(draft("Старое"), now(100))
        .await
        .expect("проект создаётся");

    let mut changed = project.clone();
    changed.rename(ProjectName::parse("Новое").expect("имя"), now(200));
    changed
        .describe("описание проекта", now(200))
        .expect("описание валидно");
    changed.recolor(ProjectColor::parse("#ABC123").expect("цвет"), now(200));
    changed.place_at(5, now(200));
    db.repo.save(&changed).await.expect("сохранение");

    let loaded = db
        .repo
        .get(project.id())
        .await
        .expect("чтение")
        .expect("проект существует");

    assert_eq!(loaded, changed);
    assert_eq!(loaded.created_at(), now(100));
    assert_eq!(loaded.updated_at(), now(200));
}

#[tokio::test]
async fn save_missing_project_reports_conflict() {
    let db = db().await;
    let missing = Project::create(ProjectId::from_raw(999), draft("Призрак"), 0, now(100));

    let error = db
        .repo
        .save(&missing)
        .await
        .expect_err("сохранение в пустую таблицу должно падать");

    assert!(matches!(error, RepositoryError::Conflict(_)));
}

#[tokio::test]
async fn corrupted_row_is_reported_as_corrupted_not_unavailable() {
    let db = db().await;
    sqlx::query(
        "INSERT INTO projects (name, description, color, position, archived, created_at, updated_at)
         VALUES ('Проект', '', 'red', 0, 0, 1, 1)",
    )
    .execute(&db.pool)
    .await
    .expect("строка вставлена в обход репозитория");

    let error = db
        .repo
        .get(ProjectId::from_raw(1))
        .await
        .expect_err("битый цвет должен отвергаться");

    assert!(matches!(error, RepositoryError::Corrupted(_)));
}

#[tokio::test]
async fn list_follows_positions_after_reorder() {
    let db = db().await;
    db.repo
        .create(draft("Первый"), now(100))
        .await
        .expect("проект создаётся");
    db.repo
        .create(draft("Второй"), now(100))
        .await
        .expect("проект создаётся");
    let mut third = db
        .repo
        .create(draft("Третий"), now(100))
        .await
        .expect("проект создаётся");

    third.place_at(-1, now(200));
    db.repo.save(&third).await.expect("перенос сохраняется");

    let names: Vec<String> = db
        .repo
        .list(false)
        .await
        .expect("список читается")
        .iter()
        .map(|project| project.name().as_str().to_owned())
        .collect();

    assert_eq!(names, ["Третий", "Первый", "Второй"]);
}
