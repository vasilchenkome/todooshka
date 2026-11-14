use crate::error::DomainError;
use crate::task::TaskId;
use crate::time::Timestamp;

pub const MAX_COMMENT_CHARS: usize = 4000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CommentId(i64);

impl CommentId {
    pub const fn from_raw(value: i64) -> Self {
        Self(value)
    }

    pub const fn as_raw(self) -> i64 {
        self.0
    }
}

impl std::fmt::Display for CommentId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommentBody(String);

impl CommentBody {
    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(DomainError::EmptyComment);
        }
        if trimmed.chars().count() > MAX_COMMENT_CHARS {
            return Err(DomainError::CommentTooLong {
                max: MAX_COMMENT_CHARS,
            });
        }
        Ok(Self(trimmed.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CommentSnapshot {
    pub id: CommentId,
    pub task_id: TaskId,
    pub body: CommentBody,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// A task comment: text and time; the author is always the same user.
#[derive(Clone, Debug, PartialEq)]
pub struct Comment {
    id: CommentId,
    task_id: TaskId,
    body: CommentBody,
    created_at: Timestamp,
    updated_at: Timestamp,
}

impl Comment {
    pub fn create(id: CommentId, task_id: TaskId, body: CommentBody, now: Timestamp) -> Self {
        Self {
            id,
            task_id,
            body,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn restore(snapshot: CommentSnapshot) -> Self {
        Self {
            id: snapshot.id,
            task_id: snapshot.task_id,
            body: snapshot.body,
            created_at: snapshot.created_at,
            updated_at: snapshot.updated_at,
        }
    }

    pub fn edit(&mut self, body: CommentBody, now: Timestamp) {
        if self.body != body {
            self.body = body;
            self.updated_at = now;
        }
    }

    pub fn id(&self) -> CommentId {
        self.id
    }

    pub fn task_id(&self) -> TaskId {
        self.task_id
    }

    pub fn body(&self) -> &CommentBody {
        &self.body
    }

    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    pub fn updated_at(&self) -> Timestamp {
        self.updated_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now(seconds: i64) -> Timestamp {
        Timestamp::from_unix_seconds(seconds)
    }

    #[test]
    fn body_is_trimmed_and_limited() {
        assert_eq!(
            CommentBody::parse("  текст  ")
                .expect("комментарий")
                .as_str(),
            "текст"
        );
        assert_eq!(
            CommentBody::parse("   ").expect_err("пусто"),
            DomainError::EmptyComment
        );
        let long = "я".repeat(MAX_COMMENT_CHARS + 1);
        assert_eq!(
            CommentBody::parse(&long).expect_err("длинно"),
            DomainError::CommentTooLong {
                max: MAX_COMMENT_CHARS
            }
        );
    }

    #[test]
    fn edit_updates_time_only_on_change() {
        let mut comment = Comment::create(
            CommentId::from_raw(1),
            TaskId::from_raw(10),
            CommentBody::parse("первый").expect("текст"),
            now(100),
        );

        comment.edit(CommentBody::parse("второй").expect("текст"), now(200));
        assert_eq!(comment.body().as_str(), "второй");
        assert_eq!(comment.updated_at(), now(200));

        comment.edit(CommentBody::parse("второй").expect("текст"), now(300));
        assert_eq!(comment.updated_at(), now(200));
    }
}
