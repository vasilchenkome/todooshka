use crate::error::DomainError;
use crate::planning::Planning;
use crate::project::ProjectId;
use crate::time::Timestamp;

/// Bounds for the task title and description.
pub const MAX_TASK_TITLE_CHARS: usize = 200;
pub const MAX_TASK_DESCRIPTION_CHARS: usize = 2000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TaskId(i64);

impl TaskId {
    pub const fn from_raw(value: i64) -> Self {
        Self(value)
    }

    pub const fn as_raw(self) -> i64 {
        self.0
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

/// Board columns: status semantics only, no UI labels or colours.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskStatus {
    Backlog,
    InProgress,
    Done,
}

impl TaskStatus {
    pub const ALL: [Self; 3] = [Self::Backlog, Self::InProgress, Self::Done];

    pub const fn is_done(self) -> bool {
        matches!(self, Self::Done)
    }

    /// The status code used in the DB and API: the only place these strings exist.
    pub const fn as_code(self) -> &'static str {
        match self {
            Self::Backlog => "backlog",
            Self::InProgress => "in_progress",
            Self::Done => "done",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "backlog" => Some(Self::Backlog),
            "in_progress" => Some(Self::InProgress),
            "done" => Some(Self::Done),
            _ => None,
        }
    }

    /// Column order on the board.
    pub const fn order(self) -> usize {
        match self {
            Self::Backlog => 0,
            Self::InProgress => 1,
            Self::Done => 2,
        }
    }
}

/// Task title: trimmed and non-empty.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskTitle(String);

impl TaskTitle {
    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(DomainError::EmptyTaskTitle);
        }
        if trimmed.chars().count() > MAX_TASK_TITLE_CHARS {
            return Err(DomainError::TaskTitleTooLong {
                max: MAX_TASK_TITLE_CHARS,
            });
        }
        Ok(Self(trimmed.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A validated draft of a new task.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskDraft {
    title: TaskTitle,
    description: String,
}

impl TaskDraft {
    pub fn new(raw_title: &str, raw_description: &str) -> Result<Self, DomainError> {
        Ok(Self {
            title: TaskTitle::parse(raw_title)?,
            description: validate_description(raw_description)?,
        })
    }

    pub fn title(&self) -> &TaskTitle {
        &self.title
    }

    pub fn description(&self) -> &str {
        &self.description
    }
}

/// Task state as read from storage: only for restoring the entity.
#[derive(Clone, Debug, PartialEq)]
pub struct TaskSnapshot {
    pub id: TaskId,
    pub project_id: ProjectId,
    pub title: TaskTitle,
    pub description: String,
    pub status: TaskStatus,
    pub position: f64,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub completed_at: Option<Timestamp>,
    pub planning: Planning,
}

/// A task on the board: belongs to a project, sits in a column at a fractional position.
#[derive(Clone, Debug, PartialEq)]
pub struct Task {
    id: TaskId,
    project_id: ProjectId,
    title: TaskTitle,
    description: String,
    status: TaskStatus,
    position: f64,
    created_at: Timestamp,
    updated_at: Timestamp,
    completed_at: Option<Timestamp>,
    planning: Planning,
}

impl Task {
    pub fn create(
        id: TaskId,
        project_id: ProjectId,
        draft: TaskDraft,
        status: TaskStatus,
        position: f64,
        now: Timestamp,
    ) -> Result<Self, DomainError> {
        ensure_position(position)?;

        Ok(Self {
            id,
            project_id,
            title: draft.title,
            description: draft.description,
            status,
            position,
            created_at: now,
            updated_at: now,
            completed_at: status.is_done().then_some(now),
            planning: Planning::default(),
        })
    }

    /// Restores a task from the DB; a status/`completed_at` mismatch means corrupted data.
    pub fn restore(snapshot: TaskSnapshot) -> Result<Self, DomainError> {
        ensure_position(snapshot.position)?;

        let description = validate_description(&snapshot.description)?;
        if snapshot.status.is_done() != snapshot.completed_at.is_some() {
            return Err(DomainError::TaskCompletionInconsistent);
        }

        Ok(Self {
            id: snapshot.id,
            project_id: snapshot.project_id,
            title: snapshot.title,
            description,
            status: snapshot.status,
            position: snapshot.position,
            created_at: snapshot.created_at,
            updated_at: snapshot.updated_at,
            completed_at: snapshot.completed_at,
            planning: snapshot.planning,
        })
    }

    pub fn rename(&mut self, title: TaskTitle, now: Timestamp) {
        self.title = title;
        self.updated_at = now;
    }

    pub fn describe(&mut self, description: &str, now: Timestamp) -> Result<(), DomainError> {
        self.description = validate_description(description)?;
        self.updated_at = now;
        Ok(())
    }

    pub fn plan(&mut self, planning: Planning, now: Timestamp) {
        self.planning = planning;
        self.updated_at = now;
    }

    /// A move between columns and within a column in one action.
    ///
    /// Entering `Done` sets `completed_at`, leaving it clears the value; moving into
    /// `Done` again keeps the original completion moment.
    pub fn move_to(
        &mut self,
        status: TaskStatus,
        position: f64,
        now: Timestamp,
    ) -> Result<(), DomainError> {
        ensure_position(position)?;

        if status.is_done() {
            if self.completed_at.is_none() {
                self.completed_at = Some(now);
            }
        } else {
            self.completed_at = None;
        }

        self.status = status;
        self.position = position;
        self.updated_at = now;
        Ok(())
    }

    /// A move within a column changes only the position, not the status or timestamps.
    pub fn place_at(&mut self, position: f64) -> Result<(), DomainError> {
        ensure_position(position)?;
        self.position = position;
        Ok(())
    }

    pub fn id(&self) -> TaskId {
        self.id
    }

    pub fn project_id(&self) -> ProjectId {
        self.project_id
    }

    pub fn title(&self) -> &TaskTitle {
        &self.title
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn status(&self) -> TaskStatus {
        self.status
    }

    pub fn position(&self) -> f64 {
        self.position
    }

    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    pub fn updated_at(&self) -> Timestamp {
        self.updated_at
    }

    pub fn completed_at(&self) -> Option<Timestamp> {
        self.completed_at
    }

    pub fn planning(&self) -> &Planning {
        &self.planning
    }
}

fn ensure_position(position: f64) -> Result<(), DomainError> {
    if position.is_finite() {
        Ok(())
    } else {
        Err(DomainError::InvalidTaskPosition)
    }
}

fn validate_description(raw: &str) -> Result<String, DomainError> {
    if raw.chars().count() > MAX_TASK_DESCRIPTION_CHARS {
        return Err(DomainError::TaskDescriptionTooLong {
            max: MAX_TASK_DESCRIPTION_CHARS,
        });
    }
    Ok(raw.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now(seconds: i64) -> Timestamp {
        Timestamp::from_unix_seconds(seconds)
    }

    fn draft(title: &str) -> TaskDraft {
        TaskDraft::new(title, "").expect("черновик валиден")
    }

    fn task(status: TaskStatus) -> Task {
        Task::create(
            TaskId::from_raw(1),
            ProjectId::from_raw(10),
            draft("Задача"),
            status,
            1.0,
            now(100),
        )
        .expect("задача создаётся")
    }

    #[test]
    fn draft_trims_title() {
        let draft = TaskDraft::new("  Сделать релиз  ", "описание").expect("черновик валиден");

        assert_eq!(draft.title().as_str(), "Сделать релиз");
    }

    #[test]
    fn blank_title_is_rejected() {
        let error = TaskDraft::new("   ", "").expect_err("пустой заголовок недопустим");

        assert_eq!(error, DomainError::EmptyTaskTitle);
    }

    #[test]
    fn title_length_counts_characters() {
        let ok = "я".repeat(MAX_TASK_TITLE_CHARS);
        let too_long = "я".repeat(MAX_TASK_TITLE_CHARS + 1);

        assert!(TaskDraft::new(&ok, "").is_ok());
        assert_eq!(
            TaskDraft::new(&too_long, "").expect_err("длинный заголовок недопустим"),
            DomainError::TaskTitleTooLong {
                max: MAX_TASK_TITLE_CHARS
            }
        );
    }

    #[test]
    fn description_length_is_limited() {
        let too_long = "я".repeat(MAX_TASK_DESCRIPTION_CHARS + 1);

        assert_eq!(
            TaskDraft::new("Задача", &too_long).expect_err("длинное описание недопустимо"),
            DomainError::TaskDescriptionTooLong {
                max: MAX_TASK_DESCRIPTION_CHARS
            }
        );
    }

    #[test]
    fn non_finite_position_is_rejected() {
        for position in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let error = Task::create(
                TaskId::from_raw(1),
                ProjectId::from_raw(10),
                draft("Задача"),
                TaskStatus::Backlog,
                position,
                now(100),
            )
            .expect_err("позиция должна быть конечной");

            assert_eq!(error, DomainError::InvalidTaskPosition);
        }
    }

    #[test]
    fn creating_done_task_sets_completion_time() {
        let task = task(TaskStatus::Done);

        assert_eq!(task.completed_at(), Some(now(100)));
    }

    #[test]
    fn move_to_done_sets_completion_time_and_backlog_clears_it() {
        let mut task = task(TaskStatus::Backlog);

        task.move_to(TaskStatus::Done, 2.0, now(200))
            .expect("перенос в Done");
        assert_eq!(task.completed_at(), Some(now(200)));
        assert_eq!(task.updated_at(), now(200));
        assert_eq!(task.position(), 2.0);

        task.move_to(TaskStatus::InProgress, 3.0, now(300))
            .expect("перенос обратно");
        assert_eq!(task.completed_at(), None);
        assert_eq!(task.status(), TaskStatus::InProgress);
    }

    #[test]
    fn repeated_move_to_done_keeps_first_completion_time() {
        let mut task = task(TaskStatus::Done);

        task.move_to(TaskStatus::Done, 5.0, now(500))
            .expect("повторный перенос");

        assert_eq!(
            task.completed_at(),
            Some(now(100)),
            "момент завершения не должен переписываться"
        );
    }

    #[test]
    fn restore_rejects_status_completion_mismatch() {
        let snapshot = TaskSnapshot {
            id: TaskId::from_raw(1),
            project_id: ProjectId::from_raw(10),
            title: TaskTitle::parse("Задача").expect("заголовок"),
            description: String::new(),
            status: TaskStatus::Done,
            position: 1.0,
            created_at: now(100),
            updated_at: now(100),
            completed_at: None,
            planning: Planning::default(),
        };

        assert_eq!(
            Task::restore(snapshot).expect_err("расхождение недопустимо"),
            DomainError::TaskCompletionInconsistent
        );
    }

    #[test]
    fn planning_changes_touch_updated_at_and_keep_status() {
        let mut task = task(TaskStatus::Backlog);
        let planning = Planning::new(
            Some(crate::DueDate::parse("2026-11-20").expect("дата")),
            crate::Priority::High,
            Some(crate::EstimateMinutes::parse(90).expect("оценка")),
        );

        task.plan(planning.clone(), now(400));

        assert_eq!(task.planning(), &planning);
        assert_eq!(task.updated_at(), now(400));
        assert_eq!(task.completed_at(), None);
        assert_eq!(task.status(), TaskStatus::Backlog);
    }

    #[test]
    fn restore_keeps_planning() {
        let planning = Planning::new(
            Some(crate::DueDate::parse("2026-12-01").expect("дата")),
            crate::Priority::Urgent,
            None,
        );
        let snapshot = TaskSnapshot {
            id: TaskId::from_raw(1),
            project_id: ProjectId::from_raw(10),
            title: TaskTitle::parse("Задача").expect("заголовок"),
            description: String::new(),
            status: TaskStatus::Backlog,
            position: 1.0,
            created_at: now(100),
            updated_at: now(100),
            completed_at: None,
            planning: planning.clone(),
        };

        assert_eq!(
            Task::restore(snapshot).expect("восстановление").planning(),
            &planning
        );
    }

    #[test]
    fn status_codes_roundtrip() {
        for status in TaskStatus::ALL {
            assert_eq!(TaskStatus::from_code(status.as_code()), Some(status));
        }
        assert_eq!(TaskStatus::from_code("archived"), None);
    }

    #[test]
    fn board_order_follows_columns() {
        assert_eq!(
            TaskStatus::ALL.map(TaskStatus::order),
            [0, 1, 2],
            "колонки должны идти слева направо"
        );
    }
}
