use crate::error::DomainError;
use crate::task::TaskId;

pub const MAX_TAG_CHARS: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TagId(i64);

impl TagId {
    pub const fn from_raw(value: i64) -> Self {
        Self(value)
    }

    pub const fn as_raw(self) -> i64 {
        self.0
    }
}

/// Имя тега: без ведущей решётки, обрезано, до 32 символов.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TagName(String);

impl TagName {
    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        let trimmed = raw.trim().trim_start_matches('#').trim();
        if trimmed.is_empty() {
            return Err(DomainError::EmptyTag);
        }
        if trimmed.chars().count() > MAX_TAG_CHARS {
            return Err(DomainError::TagTooLong { max: MAX_TAG_CHARS });
        }
        Ok(Self(trimmed.to_lowercase()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tag {
    id: TagId,
    name: TagName,
}

impl Tag {
    pub fn restore(id: TagId, name: TagName) -> Result<Self, DomainError> {
        Ok(Self { id, name })
    }

    pub fn id(&self) -> TagId {
        self.id
    }

    pub fn name(&self) -> &TagName {
        &self.name
    }
}

/// Связь тега с задачей: доска получает все связи проекта одним запросом.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskTag {
    pub task_id: TaskId,
    pub name: TagName,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tag_name_is_trimmed_and_hash_is_optional() {
        assert_eq!(TagName::parse("  #Банк  ").expect("тег").as_str(), "банк");
        assert_eq!(TagName::parse("релиз").expect("тег").as_str(), "релиз");
    }

    #[test]
    fn empty_and_long_tags_are_rejected() {
        assert_eq!(
            TagName::parse(" # ").expect_err("пусто"),
            DomainError::EmptyTag
        );

        let long = "я".repeat(MAX_TAG_CHARS + 1);
        assert_eq!(
            TagName::parse(&long).expect_err("длинный"),
            DomainError::TagTooLong { max: MAX_TAG_CHARS }
        );
    }
}
