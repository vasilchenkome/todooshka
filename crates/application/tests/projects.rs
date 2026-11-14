use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use todooshka_app::{
    ApplicationError, Clock, CreateProject, ProjectRepository, ProjectService, RepositoryError,
    UpdateProject,
};
use todooshka_domain::{DomainError, Project, ProjectDraft, ProjectId, Timestamp};

#[derive(Default)]
struct FakeProjects {
    items: Mutex<Vec<Project>>,
    next_id: AtomicI64,
    fail: AtomicBool,
    saves: AtomicI64,
}

impl FakeProjects {
    fn stored(&self) -> Vec<Project> {
        self.items.lock().expect("мьютекс не отравлен").clone()
    }

    fn fail_next(&self) {
        self.fail.store(true, Ordering::SeqCst);
    }

    fn saves_count(&self) -> i64 {
        self.saves.load(Ordering::SeqCst)
    }

    fn check_available(&self) -> Result<(), RepositoryError> {
        if self.fail.load(Ordering::SeqCst) {
            return Err(RepositoryError::Unavailable("база недоступна".to_owned()));
        }
        Ok(())
    }

    fn next_position(&self) -> i32 {
        self.stored()
            .iter()
            .map(|project| project.position())
            .max()
            .map_or(0, |max| max + 1)
    }
}

#[async_trait]
impl ProjectRepository for FakeProjects {
    async fn list(&self, include_archived: bool) -> Result<Vec<Project>, RepositoryError> {
        self.check_available()?;

        let mut items: Vec<Project> = self
            .stored()
            .into_iter()
            .filter(|project| include_archived || !project.is_archived())
            .collect();
        items.sort_by_key(|project| (project.position(), project.id().as_raw()));
        Ok(items)
    }

    async fn get(&self, id: ProjectId) -> Result<Option<Project>, RepositoryError> {
        self.check_available()?;
        Ok(self.stored().into_iter().find(|project| project.id() == id))
    }

    async fn create(
        &self,
        draft: ProjectDraft,
        now: Timestamp,
    ) -> Result<Project, RepositoryError> {
        self.check_available()?;

        let id = ProjectId::from_raw(self.next_id.fetch_add(1, Ordering::SeqCst) + 1);
        let project = Project::create(id, draft, self.next_position(), now);
        self.items
            .lock()
            .expect("мьютекс не отравлен")
            .push(project.clone());
        Ok(project)
    }

    async fn save(&self, project: &Project) -> Result<(), RepositoryError> {
        self.check_available()?;

        let mut items = self.items.lock().expect("мьютекс не отравлен");
        let Some(slot) = items.iter_mut().find(|stored| stored.id() == project.id()) else {
            return Err(RepositoryError::Unavailable(
                "проект исчез между чтением и записью".to_owned(),
            ));
        };
        *slot = project.clone();
        self.saves.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
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

struct Harness {
    projects: Arc<FakeProjects>,
    clock: Arc<TestClock>,
    service: ProjectService,
}

fn harness() -> Harness {
    let projects = Arc::new(FakeProjects::default());
    let clock = Arc::new(TestClock(AtomicI64::new(100)));

    let repository = Arc::clone(&projects) as Arc<dyn ProjectRepository>;
    let time = Arc::clone(&clock) as Arc<dyn Clock>;
    let service = ProjectService::new(repository, time);

    Harness {
        projects,
        clock,
        service,
    }
}

fn create_command(name: &str) -> CreateProject {
    CreateProject {
        name: name.to_owned(),
        description: String::new(),
        color: None,
    }
}

fn update_command(id: ProjectId) -> UpdateProject {
    UpdateProject {
        id,
        name: None,
        description: None,
        color: None,
        archived: None,
    }
}

#[tokio::test]
async fn create_assigns_ids_and_positions_in_order() {
    let h = harness();

    let first = h
        .service
        .create(create_command("Первый"))
        .await
        .expect("первый проект создаётся");
    let second = h
        .service
        .create(create_command("Второй"))
        .await
        .expect("второй проект создаётся");

    assert_eq!(first.id().as_raw(), 1);
    assert_eq!(second.id().as_raw(), 2);
    assert_eq!(first.position(), 0);
    assert_eq!(second.position(), 1);
    assert_eq!(h.projects.stored().len(), 2);
}

#[tokio::test]
async fn invalid_name_never_reaches_storage() {
    let h = harness();

    let error = h
        .service
        .create(CreateProject {
            name: "   ".to_owned(),
            description: String::new(),
            color: None,
        })
        .await
        .expect_err("пустое имя должно падать");

    assert_eq!(
        error,
        ApplicationError::Domain(DomainError::EmptyProjectName)
    );
    assert!(
        h.projects.stored().is_empty(),
        "хранилище не должно получить невалидный проект"
    );
}

#[tokio::test]
async fn list_hides_archived_until_asked() {
    let h = harness();
    let kept = h
        .service
        .create(create_command("Оставить"))
        .await
        .expect("проект создаётся");
    let archived = h
        .service
        .create(create_command("В архив"))
        .await
        .expect("проект создаётся");

    h.clock.set(200);
    let mut archive = update_command(archived.id());
    archive.archived = Some(true);
    h.service.update(archive).await.expect("архив");

    let active = h.service.list(false).await.expect("список активных");
    let all = h.service.list(true).await.expect("список всех");

    assert_eq!(
        active
            .iter()
            .map(|project| project.id())
            .collect::<Vec<_>>(),
        vec![kept.id()]
    );
    assert_eq!(all.len(), 2);
}

#[tokio::test]
async fn update_renames_project_and_bumps_updated_at() {
    let h = harness();
    let project = h
        .service
        .create(create_command("Старое"))
        .await
        .expect("проект создаётся");
    h.clock.set(200);

    let mut command = update_command(project.id());
    command.name = Some("  Новое  ".to_owned());
    let updated = h.service.update(command).await.expect("имя обновляется");

    assert_eq!(updated.name().as_str(), "Новое");
    assert_eq!(updated.updated_at(), Timestamp::from_unix_seconds(200));
    assert_eq!(updated.created_at(), Timestamp::from_unix_seconds(100));
    assert_eq!(h.projects.stored()[0].name().as_str(), "Новое");
    assert_eq!(h.projects.saves_count(), 1);
}

#[tokio::test]
async fn update_with_unchanged_values_does_not_touch_storage() {
    let h = harness();
    let project = h
        .service
        .create(create_command("Проект"))
        .await
        .expect("проект создаётся");
    h.clock.set(200);

    let mut command = update_command(project.id());
    command.name = Some("Проект".to_owned());
    command.archived = Some(false);
    let updated = h.service.update(command).await.expect("правка проходит");

    assert_eq!(updated.updated_at(), Timestamp::from_unix_seconds(100));
    assert_eq!(h.projects.saves_count(), 0, "лишней записи быть не должно");
}

#[tokio::test]
async fn update_missing_project_reports_not_found() {
    let h = harness();
    let id = ProjectId::from_raw(42);

    let error = h
        .service
        .update(update_command(id))
        .await
        .expect_err("несуществующий проект нельзя обновить");

    assert_eq!(error, ApplicationError::ProjectNotFound(id));
}

#[tokio::test]
async fn update_archive_is_idempotent() {
    let h = harness();
    let project = h
        .service
        .create(create_command("Проект"))
        .await
        .expect("проект создаётся");

    h.clock.set(200);
    let mut command = update_command(project.id());
    command.archived = Some(true);
    let first = h.service.update(command.clone()).await.expect("архив");
    h.clock.set(300);
    let second = h.service.update(command).await.expect("повторный архив");

    assert!(first.is_archived());
    assert_eq!(
        second.updated_at(),
        Timestamp::from_unix_seconds(200),
        "повторный архив не должен трогать метку"
    );
}

#[tokio::test]
async fn update_unarchive_returns_project_to_board() {
    let h = harness();
    let project = h
        .service
        .create(create_command("Проект"))
        .await
        .expect("проект создаётся");

    let mut archive = update_command(project.id());
    archive.archived = Some(true);
    h.service.update(archive).await.expect("архив");

    h.clock.set(200);
    let mut unarchive = update_command(project.id());
    unarchive.archived = Some(false);
    let restored = h.service.update(unarchive).await.expect("возврат");

    assert!(!restored.is_archived());
    assert_eq!(h.service.list(false).await.expect("список").len(), 1);
}

#[tokio::test]
async fn storage_failure_is_reported_not_hidden() {
    let h = harness();
    h.projects.fail_next();

    let error = h
        .service
        .create(create_command("Проект"))
        .await
        .expect_err("отказ хранилища должен всплыть");

    assert!(matches!(
        error,
        ApplicationError::Repository(RepositoryError::Unavailable(_))
    ));
}
