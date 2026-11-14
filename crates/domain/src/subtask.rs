use crate::error::DomainError;
use crate::task::TaskId;
use crate::time::Timestamp;

pub const MAX_SUBTASK_TITLE_CHARS: usize = 200;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SubtaskId(i64);

impl SubtaskId {
    pub const fn from_raw(value: i64) -> Self {
        Self(value)
    }

    pub const fn as_raw(self) -> i64 {
        self.0
    }
}

impl std::fmt::Display for SubtaskId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubtaskTitle(String);

impl SubtaskTitle {
    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(DomainError::EmptySubtaskTitle);
        }
        if trimmed.chars().count() > MAX_SUBTASK_TITLE_CHARS {
            return Err(DomainError::SubtaskTitleTooLong {
                max: MAX_SUBTASK_TITLE_CHARS,
            });
        }
        Ok(Self(trimmed.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SubtaskSnapshot {
    pub id: SubtaskId,
    pub task_id: TaskId,
    pub title: SubtaskTitle,
    pub done: bool,
    pub position: f64,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// A checklist item: a short line, a tick and its order inside the task.
#[derive(Clone, Debug, PartialEq)]
pub struct Subtask {
    id: SubtaskId,
    task_id: TaskId,
    title: SubtaskTitle,
    done: bool,
    position: f64,
    created_at: Timestamp,
    updated_at: Timestamp,
}

impl Subtask {
    pub fn create(
        id: SubtaskId,
        task_id: TaskId,
        title: SubtaskTitle,
        position: f64,
        now: Timestamp,
    ) -> Result<Self, DomainError> {
        ensure_position(position)?;

        Ok(Self {
            id,
            task_id,
            title,
            done: false,
            position,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn restore(snapshot: SubtaskSnapshot) -> Result<Self, DomainError> {
        ensure_position(snapshot.position)?;

        Ok(Self {
            id: snapshot.id,
            task_id: snapshot.task_id,
            title: snapshot.title,
            done: snapshot.done,
            position: snapshot.position,
            created_at: snapshot.created_at,
            updated_at: snapshot.updated_at,
        })
    }

    pub fn rename(&mut self, title: SubtaskTitle, now: Timestamp) {
        if self.title != title {
            self.title = title;
            self.updated_at = now;
        }
    }

    pub fn set_done(&mut self, done: bool, now: Timestamp) {
        if self.done != done {
            self.done = done;
            self.updated_at = now;
        }
    }

    pub fn id(&self) -> SubtaskId {
        self.id
    }

    pub fn task_id(&self) -> TaskId {
        self.task_id
    }

    pub fn title(&self) -> &SubtaskTitle {
        &self.title
    }

    pub fn is_done(&self) -> bool {
        self.done
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
}

fn ensure_position(position: f64) -> Result<(), DomainError> {
    if position.is_finite() {
        Ok(())
    } else {
        Err(DomainError::InvalidTaskPosition)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now(seconds: i64) -> Timestamp {
        Timestamp::from_unix_seconds(seconds)
    }

    fn title(value: &str) -> SubtaskTitle {
        SubtaskTitle::parse(value).expect("заголовок валиден")
    }

    fn subtask() -> Subtask {
        Subtask::create(
            SubtaskId::from_raw(1),
            TaskId::from_raw(10),
            title("Проверить ретраи"),
            1.0,
            now(100),
        )
        .expect("пункт создаётся")
    }

    #[test]
    fn title_is_trimmed_and_limited() {
        assert_eq!(title("  Проверить  ").as_str(), "Проверить");
        assert_eq!(
            SubtaskTitle::parse("   ").expect_err("пустой"),
            DomainError::EmptySubtaskTitle
        );
        let too_long = "я".repeat(MAX_SUBTASK_TITLE_CHARS + 1);
        assert_eq!(
            SubtaskTitle::parse(&too_long).expect_err("длинный"),
            DomainError::SubtaskTitleTooLong {
                max: MAX_SUBTASK_TITLE_CHARS
            }
        );
    }

    #[test]
    fn done_toggle_updates_time_only_on_change() {
        let mut item = subtask();

        item.set_done(true, now(200));
        assert!(item.is_done());
        assert_eq!(item.updated_at(), now(200));

        item.set_done(true, now(300));
        assert_eq!(item.updated_at(), now(200));
    }

    #[test]
    fn rename_trims_and_touches_time() {
        let mut item = subtask();

        item.rename(title("Новое имя"), now(400));

        assert_eq!(item.title().as_str(), "Новое имя");
        assert_eq!(item.updated_at(), now(400));
    }

    #[test]
    fn non_finite_position_is_rejected() {
        let error = Subtask::create(
            SubtaskId::from_raw(1),
            TaskId::from_raw(10),
            title("Пункт"),
            f64::INFINITY,
            now(100),
        )
        .expect_err("позиция должна быть конечной");

        assert_eq!(error, DomainError::InvalidTaskPosition);
    }
}
