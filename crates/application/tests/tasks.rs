use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use todooshka_app::{
    AddTask, ApplicationError, Clock, EditTask, MoveTask, ProjectRepository, RepositoryError,
    TaskService, TaskStore, TaskTransaction,
};
use todooshka_domain::{
    DomainError, Project, ProjectDraft, ProjectId, Task, TaskDraft, TaskId, TaskStatus, Timestamp,
};

fn now(seconds: i64) -> Timestamp {
    Timestamp::from_unix_seconds(seconds)
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

struct FakeProjects {
    items: Mutex<Vec<Project>>,
}

#[async_trait]
impl ProjectRepository for FakeProjects {
    async fn list(&self, include_archived: bool) -> Result<Vec<Project>, RepositoryError> {
        Ok(self
            .items
            .lock()
            .expect("мьютекс не отравлен")
            .iter()
            .filter(|project| include_archived || !project.is_archived())
            .cloned()
            .collect())
    }

    async fn get(&self, id: ProjectId) -> Result<Option<Project>, RepositoryError> {
        Ok(self
            .items
            .lock()
            .expect("мьютекс не отравлен")
            .iter()
            .find(|project| project.id() == id)
            .cloned())
    }

    async fn create(
        &self,
        _draft: ProjectDraft,
        _now: Timestamp,
    ) -> Result<Project, RepositoryError> {
        unreachable!("проект засевается заранее")
    }

    async fn save(&self, _project: &Project) -> Result<(), RepositoryError> {
        unreachable!("проекты в этих тестах не меняются")
    }
}

#[derive(Default, Clone)]
struct FakeTasks {
    items: Arc<Mutex<Vec<Task>>>,
    next_id: Arc<AtomicI64>,
}

impl FakeTasks {
    fn stored(&self) -> Vec<Task> {
        self.items.lock().expect("мьютекс не отравлен").clone()
    }

    fn seed(&self, tasks: Vec<Task>) {
        *self.items.lock().expect("мьютекс не отравлен") = tasks;
    }

    fn next_task_id(&self) -> TaskId {
        TaskId::from_raw(self.next_id.fetch_add(1, Ordering::SeqCst) + 1)
    }

    fn column_of(&self, project_id: ProjectId, status: TaskStatus) -> Vec<Task> {
        let mut column: Vec<Task> = self
            .stored()
            .into_iter()
            .filter(|task| task.project_id() == project_id && task.status() == status)
            .collect();
        column.sort_by(|left, right| {
            left.position()
                .total_cmp(&right.position())
                .then_with(|| left.id().as_raw().cmp(&right.id().as_raw()))
        });
        column
    }

    fn next_position(&self, project_id: ProjectId, status: TaskStatus) -> f64 {
        self.column_of(project_id, status)
            .last()
            .map_or(1.0, |task| task.position() + 1.0)
    }
}

#[async_trait]
impl TaskStore for FakeTasks {
    async fn list_by_project(&self, project_id: ProjectId) -> Result<Vec<Task>, RepositoryError> {
        let tasks: Vec<Task> = self
            .stored()
            .into_iter()
            .filter(|task| task.project_id() == project_id)
            .collect();
        Ok(tasks)
    }

    async fn delete(&self, id: TaskId) -> Result<bool, RepositoryError> {
        let mut items = self.items.lock().expect("мьютекс не отравлен");
        let before = items.len();
        items.retain(|task| task.id() != id);
        Ok(items.len() != before)
    }

    async fn transaction(&self) -> Result<Box<dyn TaskTransaction>, RepositoryError> {
        Ok(Box::new(FakeTaskTransaction {
            tasks: self.clone(),
        }))
    }
}

struct FakeTaskTransaction {
    tasks: FakeTasks,
}

#[async_trait]
impl TaskTransaction for FakeTaskTransaction {
    async fn list_column(
        &mut self,
        project_id: ProjectId,
        status: TaskStatus,
    ) -> Result<Vec<Task>, RepositoryError> {
        Ok(self.tasks.column_of(project_id, status))
    }

    async fn get(&mut self, id: TaskId) -> Result<Option<Task>, RepositoryError> {
        Ok(self.tasks.stored().into_iter().find(|task| task.id() == id))
    }

    async fn create(
        &mut self,
        project_id: ProjectId,
        draft: TaskDraft,
        status: TaskStatus,
        now: Timestamp,
    ) -> Result<Task, RepositoryError> {
        let id = self.tasks.next_task_id();
        let position = self.tasks.next_position(project_id, status);
        let task = Task::create(id, project_id, draft, status, position, now)
            .map_err(|error| RepositoryError::Corrupted(error.to_string()))?;

        self.tasks
            .items
            .lock()
            .expect("мьютекс не отравлен")
            .push(task.clone());
        Ok(task)
    }

    async fn save(&mut self, task: &Task) -> Result<(), RepositoryError> {
        let mut items = self.tasks.items.lock().expect("мьютекс не отравлен");
        let Some(slot) = items.iter_mut().find(|stored| stored.id() == task.id()) else {
            return Err(RepositoryError::Conflict(
                "задача исчезла между чтением и записью".to_owned(),
            ));
        };
        *slot = task.clone();
        Ok(())
    }

    async fn commit(&mut self) -> Result<(), RepositoryError> {
        Ok(())
    }
}

struct Harness {
    tasks: FakeTasks,
    clock: Arc<TestClock>,
    service: TaskService,
    project_id: ProjectId,
}

fn harness() -> Harness {
    let project_id = ProjectId::from_raw(1);
    let project = Project::create(
        project_id,
        ProjectDraft::new("Проект", "", None).expect("проект валиден"),
        0,
        now(50),
    );

    let projects = FakeProjects {
        items: Mutex::new(vec![project]),
    };
    let tasks = FakeTasks::default();
    let clock = Arc::new(TestClock(AtomicI64::new(100)));

    let service = TaskService::new(
        Arc::new(tasks.clone()) as Arc<dyn TaskStore>,
        Arc::new(projects) as Arc<dyn ProjectRepository>,
        Arc::clone(&clock) as Arc<dyn Clock>,
    );

    Harness {
        tasks,
        clock,
        service,
        project_id,
    }
}

async fn add(h: &Harness, title: &str, status: TaskStatus) -> Task {
    h.service
        .add(AddTask {
            project_id: h.project_id,
            title: title.to_owned(),
            description: String::new(),
            status,
        })
        .await
        .expect("задача создаётся")
}

async fn move_task(h: &Harness, id: TaskId, status: TaskStatus, index: usize) -> Task {
    h.service
        .move_task(MoveTask {
            id,
            status,
            target_index: index,
        })
        .await
        .expect("перенос проходит")
}

fn titles(column: &[Task]) -> Vec<&str> {
    column.iter().map(|task| task.title().as_str()).collect()
}

#[tokio::test]
async fn add_appends_to_end_of_its_column() {
    let h = harness();

    let first = add(&h, "Первая", TaskStatus::Backlog).await;
    let second = add(&h, "Вторая", TaskStatus::Backlog).await;
    let done = add(&h, "Готово", TaskStatus::Done).await;

    assert_eq!(first.position(), 1.0);
    assert_eq!(second.position(), 2.0);
    assert_eq!(done.position(), 1.0, "колонка Done считается отдельно");
    assert_eq!(done.completed_at(), Some(now(100)));
}

#[tokio::test]
async fn add_to_missing_project_reports_not_found() {
    let h = harness();

    let error = h
        .service
        .add(AddTask {
            project_id: ProjectId::from_raw(999),
            title: "Задача".to_owned(),
            description: String::new(),
            status: TaskStatus::Backlog,
        })
        .await
        .expect_err("проекта нет");

    assert_eq!(
        error,
        ApplicationError::ProjectNotFound(ProjectId::from_raw(999))
    );
}

#[tokio::test]
async fn invalid_title_never_reaches_storage() {
    let h = harness();

    let error = h
        .service
        .add(AddTask {
            project_id: h.project_id,
            title: "   ".to_owned(),
            description: String::new(),
            status: TaskStatus::Backlog,
        })
        .await
        .expect_err("пустой заголовок недопустим");

    assert_eq!(error, ApplicationError::Domain(DomainError::EmptyTaskTitle));
    assert!(h.tasks.stored().is_empty());
}

#[tokio::test]
async fn list_orders_tasks_by_board_columns() {
    let h = harness();
    add(&h, "Готово", TaskStatus::Done).await;
    add(&h, "В работе", TaskStatus::InProgress).await;
    add(&h, "Бэклог", TaskStatus::Backlog).await;

    let tasks = h.service.list(h.project_id).await.expect("список задач");
    let statuses: Vec<TaskStatus> = tasks.iter().map(|task| task.status()).collect();

    assert_eq!(
        statuses,
        vec![
            TaskStatus::Backlog,
            TaskStatus::InProgress,
            TaskStatus::Done
        ]
    );
}

#[tokio::test]
async fn move_between_columns_sets_completion_time() {
    let h = harness();
    let task = add(&h, "Задача", TaskStatus::Backlog).await;
    h.clock.set(200);

    let moved = move_task(&h, task.id(), TaskStatus::Done, 0).await;

    assert_eq!(moved.status(), TaskStatus::Done);
    assert_eq!(moved.completed_at(), Some(now(200)));
    assert_eq!(moved.position(), 1.0);
    assert_eq!(moved.updated_at(), now(200));
}

#[tokio::test]
async fn move_back_from_done_clears_completion_time() {
    let h = harness();
    let task = add(&h, "Задача", TaskStatus::Done).await;

    let moved = move_task(&h, task.id(), TaskStatus::Backlog, 0).await;

    assert_eq!(moved.completed_at(), None);
}

#[tokio::test]
async fn move_to_top_places_task_first() {
    let h = harness();
    let first = add(&h, "Первая", TaskStatus::Backlog).await;
    add(&h, "Вторая", TaskStatus::Backlog).await;
    let third = add(&h, "Третья", TaskStatus::Backlog).await;

    move_task(&h, third.id(), TaskStatus::Backlog, 0).await;

    let column = h.tasks.column_of(h.project_id, TaskStatus::Backlog);
    assert_eq!(titles(&column), ["Третья", "Первая", "Вторая"]);

    let third = column
        .iter()
        .find(|task| task.id() == third.id())
        .expect("задача на месте");
    assert_eq!(
        third.position(),
        first.position() - 1.0,
        "вставка в начало сдвигает позицию на шаг"
    );
}

#[tokio::test]
async fn move_within_column_inserts_between_neighbours() {
    let h = harness();
    add(&h, "Первая", TaskStatus::Backlog).await;
    add(&h, "Вторая", TaskStatus::Backlog).await;
    let third = add(&h, "Третья", TaskStatus::Backlog).await;
    h.clock.set(200);

    let moved = move_task(&h, third.id(), TaskStatus::Backlog, 1).await;

    let column = h.tasks.column_of(h.project_id, TaskStatus::Backlog);
    assert_eq!(titles(&column), ["Первая", "Третья", "Вторая"]);
    assert_eq!(moved.position(), 1.5);
    assert_eq!(moved.updated_at(), now(200));
}

#[tokio::test]
async fn move_beyond_end_lands_last() {
    let h = harness();
    let first = add(&h, "Первая", TaskStatus::Backlog).await;
    add(&h, "Вторая", TaskStatus::Backlog).await;

    move_task(&h, first.id(), TaskStatus::Backlog, 99).await;

    let column = h.tasks.column_of(h.project_id, TaskStatus::Backlog);
    assert_eq!(titles(&column), ["Вторая", "Первая"]);
}

#[tokio::test]
async fn exhausted_gap_triggers_renumbering_without_touching_timestamps() {
    let h = harness();
    let first = add(&h, "Первая", TaskStatus::Backlog).await;
    let second = add(&h, "Вторая", TaskStatus::Backlog).await;
    let third = add(&h, "Третья", TaskStatus::Backlog).await;

    let mut tasks = h.tasks.stored();
    for task in &mut tasks {
        if task.id() == second.id() {
            task.place_at(first.position() + 1e-12)
                .expect("позиция конечна");
        }
    }
    h.tasks.seed(tasks);

    let moved = move_task(&h, third.id(), TaskStatus::Backlog, 1).await;

    let column = h.tasks.column_of(h.project_id, TaskStatus::Backlog);
    assert_eq!(titles(&column), ["Первая", "Третья", "Вторая"]);
    assert_eq!(
        column
            .iter()
            .map(|task| task.position())
            .collect::<Vec<_>>(),
        [1.0, 1.5, 2.0]
    );
    assert_eq!(moved.completed_at(), None);

    let first_after = column
        .iter()
        .find(|task| task.id() == first.id())
        .expect("первая на месте");
    let second_after = column
        .iter()
        .find(|task| task.id() == second.id())
        .expect("вторая на месте");
    assert_eq!(
        first_after.updated_at(),
        now(100),
        "перенумерация не должна выглядеть правкой"
    );
    assert_eq!(second_after.updated_at(), now(100));
}

#[tokio::test]
async fn move_missing_task_reports_not_found() {
    let h = harness();
    let id = TaskId::from_raw(404);

    let error = h
        .service
        .move_task(MoveTask {
            id,
            status: TaskStatus::Done,
            target_index: 0,
        })
        .await
        .expect_err("задачи нет");

    assert_eq!(error, ApplicationError::TaskNotFound(id));
}

#[tokio::test]
async fn edit_updates_title_and_description() {
    let h = harness();
    let task = add(&h, "Старинная", TaskStatus::Backlog).await;
    h.clock.set(200);

    let edited = h
        .service
        .edit(EditTask {
            id: task.id(),
            title: Some("  Новая  ".to_owned()),
            description: Some("подробности".to_owned()),
        })
        .await
        .expect("правка проходит");

    assert_eq!(edited.title().as_str(), "Новая");
    assert_eq!(edited.description(), "подробности");
    assert_eq!(edited.updated_at(), now(200));
    assert_eq!(
        h.tasks.stored()[0].title().as_str(),
        "Новая",
        "изменения должны дойти до хранилища"
    );
}

#[tokio::test]
async fn edit_with_invalid_title_changes_nothing() {
    let h = harness();
    let task = add(&h, "Старинная", TaskStatus::Backlog).await;

    let error = h
        .service
        .edit(EditTask {
            id: task.id(),
            title: Some("   ".to_owned()),
            description: Some("подробности".to_owned()),
        })
        .await
        .expect_err("пустой заголовок недопустим");

    assert_eq!(error, ApplicationError::Domain(DomainError::EmptyTaskTitle));
    assert_eq!(h.tasks.stored()[0].title().as_str(), "Старинная");
    assert_eq!(h.tasks.stored()[0].description(), "");
}

#[tokio::test]
async fn delete_removes_task_and_reports_missing() {
    let h = harness();
    let task = add(&h, "Задача", TaskStatus::Backlog).await;

    h.service.delete(task.id()).await.expect("задача удаляется");
    assert!(h.tasks.stored().is_empty());

    let error = h
        .service
        .delete(task.id())
        .await
        .expect_err("второе удаление не проходит");
    assert_eq!(error, ApplicationError::TaskNotFound(task.id()));
}
