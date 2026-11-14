use std::sync::Arc;

use todooshka_domain::{
    Planning, ProjectId, Task, TaskDraft, TaskId, TaskStatus, TaskTitle, between, evenly,
};

use crate::error::ApplicationError;
use crate::ports::{Clock, ProjectRepository, ProjectTaskCount, TaskStore, TaskTransaction};

/// Add a task to the end of the chosen column.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AddTask {
    pub project_id: ProjectId,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
}

/// Edit the task's content; fields missing from the command are left untouched.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditTask {
    pub id: TaskId,
    pub title: Option<String>,
    pub description: Option<String>,
    pub planning: Option<Planning>,
}

/// Move a task into a column at a given spot (index in the column, 0 is the top).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MoveTask {
    pub id: TaskId,
    pub status: TaskStatus,
    pub target_index: usize,
}

/// Board task use cases.
pub struct TaskService {
    tasks: Arc<dyn TaskStore>,
    projects: Arc<dyn ProjectRepository>,
    clock: Arc<dyn Clock>,
}

impl TaskService {
    pub fn new(
        tasks: Arc<dyn TaskStore>,
        projects: Arc<dyn ProjectRepository>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            tasks,
            projects,
            clock,
        }
    }

    pub async fn add(&self, command: AddTask) -> Result<Task, ApplicationError> {
        let draft = TaskDraft::new(&command.title, &command.description)?;
        self.require_project(command.project_id).await?;

        let now = self.clock.now();
        let mut transaction = self.tasks.transaction().await?;
        let task = transaction
            .create(command.project_id, draft, command.status, now)
            .await?;
        transaction.commit().await?;

        Ok(task)
    }

    pub async fn list(&self, project_id: ProjectId) -> Result<Vec<Task>, ApplicationError> {
        self.require_project(project_id).await?;

        let mut tasks = self.tasks.list_by_project(project_id).await?;
        tasks.sort_by(|left, right| {
            left.status()
                .order()
                .cmp(&right.status().order())
                .then_with(|| left.position().total_cmp(&right.position()))
                .then_with(|| left.id().as_raw().cmp(&right.id().as_raw()))
        });

        Ok(tasks)
    }

    /// Task counters for all projects: the project list uses them instead of N+1 queries.
    pub async fn counts_by_project(&self) -> Result<Vec<ProjectTaskCount>, ApplicationError> {
        Ok(self.tasks.counts_by_project().await?)
    }

    pub async fn edit(&self, command: EditTask) -> Result<Task, ApplicationError> {
        let title = command.title.as_deref().map(TaskTitle::parse).transpose()?;

        let mut transaction = self.tasks.transaction().await?;
        let mut task = transaction
            .get(command.id)
            .await?
            .ok_or(ApplicationError::TaskNotFound(command.id))?;

        let now = self.clock.now();
        let mut changed = false;

        if let Some(title) = title
            && title != *task.title()
        {
            task.rename(title, now);
            changed = true;
        }

        if let Some(description) = command.description.as_deref()
            && description != task.description()
        {
            task.describe(description, now)?;
            changed = true;
        }

        if let Some(planning) = command.planning
            && &planning != task.planning()
        {
            task.plan(planning, now);
            changed = true;
        }

        if changed {
            transaction.save(&task).await?;
        }
        transaction.commit().await?;

        Ok(task)
    }

    /// Moving a task: reading the column, picking a position and writing in one transaction.
    ///
    /// An index past the column end means "append": the expected drag&drop
    /// behaviour when a card is dropped below the last one.
    pub async fn move_task(&self, command: MoveTask) -> Result<Task, ApplicationError> {
        let now = self.clock.now();
        let mut transaction = self.tasks.transaction().await?;
        let mut task = transaction
            .get(command.id)
            .await?
            .ok_or(ApplicationError::TaskNotFound(command.id))?;

        let mut column = transaction
            .list_column(task.project_id(), command.status)
            .await?
            .into_iter()
            .filter(|candidate| candidate.id() != task.id())
            .collect::<Vec<_>>();
        column.sort_by(|left, right| {
            left.position()
                .total_cmp(&right.position())
                .then_with(|| left.id().as_raw().cmp(&right.id().as_raw()))
        });

        let index = command.target_index.min(column.len());
        let position = match position_for(&column, index) {
            Some(position) => position,
            None => {
                renumber_column(&mut *transaction, &mut column).await?;
                position_for(&column, index).expect("после перенумерации зазор всегда есть")
            }
        };

        task.move_to(command.status, position, now)?;
        transaction.save(&task).await?;
        transaction.commit().await?;

        Ok(task)
    }

    pub async fn delete(&self, id: TaskId) -> Result<(), ApplicationError> {
        if self.tasks.delete(id).await? {
            Ok(())
        } else {
            Err(ApplicationError::TaskNotFound(id))
        }
    }

    async fn require_project(&self, id: ProjectId) -> Result<(), ApplicationError> {
        if self.projects.get(id).await?.is_none() {
            return Err(ApplicationError::ProjectNotFound(id));
        }
        Ok(())
    }
}

/// Position for an insert at `index` in an already sorted column without the task itself.
fn position_for(column: &[Task], index: usize) -> Option<f64> {
    let before = index.checked_sub(1).map(|index| column[index].position());
    let after = column.get(index).map(|task| task.position());
    between(before, after)
}

/// Renumbers the column in even steps without touching timestamps:
/// it is a maintenance operation, not a user edit of the task.
async fn renumber_column(
    transaction: &mut dyn TaskTransaction,
    column: &mut [Task],
) -> Result<(), ApplicationError> {
    let positions = evenly(column.len());
    for (task, position) in column.iter_mut().zip(positions) {
        if task.position() != position {
            task.place_at(position)?;
            transaction.save(task).await?;
        }
    }
    Ok(())
}
