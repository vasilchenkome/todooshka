use async_trait::async_trait;
use todooshka_app::{AgendaEntry, AgendaReader, AgendaService, RepositoryError};
use todooshka_domain::{
    DueDate, Planning, Priority, ProjectId, Task, TaskDraft, TaskId, TaskStatus, Timestamp,
};

fn now() -> Timestamp {
    Timestamp::from_unix_seconds(1_700_000_000)
}

fn task(
    id: i64,
    project: i64,
    name: &str,
    due: Option<&str>,
    priority: Priority,
    status: TaskStatus,
) -> AgendaEntry {
    let due = due.map(|value| DueDate::parse(value).expect("валидная дата"));
    let task = Task::create(
        TaskId::from_raw(id),
        ProjectId::from_raw(project),
        TaskDraft::new(name, "").expect("валидная задача"),
        status,
        1.0,
        now(),
    )
    .expect("задача создана");
    let mut task = task;
    task.plan(Planning::new(due, priority, None), now());
    AgendaEntry {
        project_id: ProjectId::from_raw(project),
        project_name: format!("Проект {project}"),
        task,
    }
}

struct FakeAgenda(Vec<AgendaEntry>);

#[async_trait]
impl AgendaReader for FakeAgenda {
    async fn list_open_due_until(
        &self,
        through: &DueDate,
    ) -> Result<Vec<AgendaEntry>, RepositoryError> {
        Ok(self
            .0
            .iter()
            .filter(|entry| {
                entry
                    .task
                    .planning()
                    .due_on()
                    .is_some_and(|due| due <= through)
            })
            .cloned()
            .collect())
    }
}

fn service(entries: Vec<AgendaEntry>) -> AgendaService {
    AgendaService::new(std::sync::Arc::new(FakeAgenda(entries)))
}

#[tokio::test]
async fn buckets_boundaries_and_excludes_done_without_due() {
    let today = DueDate::parse("2026-11-14").expect("дата");
    let agenda = service(vec![
        task(
            1,
            1,
            "вчера",
            Some("2026-11-13"),
            Priority::Normal,
            TaskStatus::Backlog,
        ),
        task(
            2,
            1,
            "сегодня",
            Some("2026-11-14"),
            Priority::Normal,
            TaskStatus::InProgress,
        ),
        task(
            3,
            2,
            "завтра",
            Some("2026-11-15"),
            Priority::Normal,
            TaskStatus::Backlog,
        ),
        task(
            4,
            2,
            "+7",
            Some("2026-11-21"),
            Priority::Normal,
            TaskStatus::Backlog,
        ),
        task(
            5,
            1,
            "+8",
            Some("2026-11-22"),
            Priority::Normal,
            TaskStatus::Backlog,
        ),
        task(
            6,
            1,
            "без срока",
            None,
            Priority::Normal,
            TaskStatus::Backlog,
        ),
        task(
            7,
            1,
            "готово",
            Some("2026-11-14"),
            Priority::Normal,
            TaskStatus::Done,
        ),
    ])
    .build(today)
    .await
    .expect("agenda");

    assert_eq!(agenda.overdue.len(), 1);
    assert_eq!(agenda.today.len(), 1);
    assert_eq!(agenda.week.len(), 2);
    assert_eq!(agenda.overdue[0].task.title().as_str(), "вчера");
    assert_eq!(agenda.today[0].task.title().as_str(), "сегодня");
    assert_eq!(agenda.week[0].task.title().as_str(), "завтра");
    assert_eq!(agenda.week[1].task.title().as_str(), "+7");
}

#[tokio::test]
async fn sorts_by_urgency_within_the_same_due_date() {
    let today = DueDate::parse("2026-11-14").expect("дата");
    let agenda = service(vec![
        task(
            1,
            1,
            "низкий",
            Some("2026-11-14"),
            Priority::Low,
            TaskStatus::Backlog,
        ),
        task(
            2,
            1,
            "срочный",
            Some("2026-11-14"),
            Priority::Urgent,
            TaskStatus::Backlog,
        ),
        task(
            3,
            1,
            "высокий",
            Some("2026-11-14"),
            Priority::High,
            TaskStatus::Backlog,
        ),
    ])
    .build(today)
    .await
    .expect("agenda");

    assert_eq!(
        agenda
            .today
            .iter()
            .map(|entry| entry.task.title().as_str())
            .collect::<Vec<_>>(),
        ["срочный", "высокий", "низкий"]
    );
}

#[tokio::test]
async fn empty_agenda_has_empty_sections() {
    let today = DueDate::parse("2026-11-14").expect("дата");
    let agenda = service(Vec::new()).build(today).await.expect("agenda");
    assert!(agenda.overdue.is_empty());
    assert!(agenda.today.is_empty());
    assert!(agenda.week.is_empty());
}
