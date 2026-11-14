use std::sync::Arc;

use todooshka_domain::{DueDate, ProjectId, Task};

use crate::error::ApplicationError;
use crate::ports::AgendaReader;

/// One agenda row: the task plus the project it belongs to.
#[derive(Clone, Debug, PartialEq)]
pub struct AgendaEntry {
    pub project_id: ProjectId,
    pub project_name: String,
    pub task: Task,
}

/// The three read-only sections of the Today screen.
#[derive(Clone, Debug, PartialEq)]
pub struct Agenda {
    pub date: DueDate,
    pub overdue: Vec<AgendaEntry>,
    pub today: Vec<AgendaEntry>,
    pub week: Vec<AgendaEntry>,
}

/// Read-only scenario behind the Today screen.
pub struct AgendaService {
    reader: Arc<dyn AgendaReader>,
}

impl AgendaService {
    pub fn new(reader: Arc<dyn AgendaReader>) -> Self {
        Self { reader }
    }

    /// Buckets open tasks by the caller's calendar date:
    /// overdue `< today`, today `== today`, week `today + 1 … today + 7`.
    pub async fn build(&self, today: DueDate) -> Result<Agenda, ApplicationError> {
        let week_end = today.plus_days(7);
        let through = week_end.clone().unwrap_or_else(|| today.clone());
        let entries = self.reader.list_open_due_until(&through).await?;

        let mut overdue = Vec::new();
        let mut due_today = Vec::new();
        let mut week = Vec::new();

        for entry in entries {
            if entry.task.status().is_done() {
                continue;
            }
            let Some(due) = entry.task.planning().due_on() else {
                continue;
            };

            if due < &today {
                overdue.push(entry);
            } else if due == &today {
                due_today.push(entry);
            } else if week_end.as_ref().is_some_and(|end| due <= end) {
                week.push(entry);
            }
        }

        sort_section(&mut overdue);
        sort_section(&mut due_today);
        sort_section(&mut week);

        Ok(Agenda {
            date: today,
            overdue,
            today: due_today,
            week,
        })
    }
}

/// Due date first, then urgency: urgent before high before normal before low.
fn sort_section(entries: &mut [AgendaEntry]) {
    entries.sort_by(|left, right| {
        left.task
            .planning()
            .due_on()
            .cmp(&right.task.planning().due_on())
            .then_with(|| {
                right
                    .task
                    .planning()
                    .priority()
                    .cmp(&left.task.planning().priority())
            })
            .then_with(|| left.project_name.cmp(&right.project_name))
            .then_with(|| left.task.id().as_raw().cmp(&right.task.id().as_raw()))
    });
}
