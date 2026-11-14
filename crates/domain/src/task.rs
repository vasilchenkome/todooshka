/// Идентификатор задачи; задачи появятся в следующем шаге, статус нужен уже сейчас.
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

/// Колонки доски: семантика статусов, без подписей и цветов интерфейса.
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_done_status_is_done() {
        assert!(TaskStatus::Done.is_done());
        assert!(!TaskStatus::Backlog.is_done());
        assert!(!TaskStatus::InProgress.is_done());
    }

    #[test]
    fn all_lists_board_columns_in_order() {
        assert_eq!(
            TaskStatus::ALL,
            [
                TaskStatus::Backlog,
                TaskStatus::InProgress,
                TaskStatus::Done
            ]
        );
    }
}
