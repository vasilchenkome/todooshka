use crate::error::DomainError;
use crate::time::Timestamp;

/// Границы имени и описания: длиннее уже не читается на доске, короче — не говорим ничего.
pub const MAX_PROJECT_NAME_CHARS: usize = 80;
pub const MAX_PROJECT_DESCRIPTION_CHARS: usize = 2000;

/// Цвет по умолчанию для проектов без выбранного цвета.
pub const DEFAULT_PROJECT_COLOR: &str = "#5f6b76";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ProjectId(i64);

impl ProjectId {
    pub const fn from_raw(value: i64) -> Self {
        Self(value)
    }

    pub const fn as_raw(self) -> i64 {
        self.0
    }
}

/// Имя проекта уже обрезано по краям и проверено; пустым его сделать нельзя.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectName(String);

impl ProjectName {
    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(DomainError::EmptyProjectName);
        }
        if trimmed.chars().count() > MAX_PROJECT_NAME_CHARS {
            return Err(DomainError::ProjectNameTooLong {
                max: MAX_PROJECT_NAME_CHARS,
            });
        }
        Ok(Self(trimmed.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Цвет строго в `#RRGGBB` и в нижнем регистре, чтобы сравнение было предсказуемым.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectColor(String);

impl ProjectColor {
    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        let value = raw.trim();
        let is_hex = value.len() == 7
            && value.starts_with('#')
            && value[1..].chars().all(|c| c.is_ascii_hexdigit());

        if !is_hex {
            return Err(DomainError::InvalidProjectColor(raw.to_owned()));
        }

        Ok(Self(value.to_ascii_lowercase()))
    }

    fn default_color() -> Self {
        Self(DEFAULT_PROJECT_COLOR.to_owned())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Проверенный черновик нового проекта: всё, что известно до появления id.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectDraft {
    name: ProjectName,
    description: String,
    color: ProjectColor,
}

impl ProjectDraft {
    pub fn new(
        raw_name: &str,
        raw_description: &str,
        raw_color: Option<&str>,
    ) -> Result<Self, DomainError> {
        Ok(Self {
            name: ProjectName::parse(raw_name)?,
            description: validate_description(raw_description)?,
            color: match raw_color {
                Some(color) => ProjectColor::parse(color)?,
                None => ProjectColor::default_color(),
            },
        })
    }

    pub fn name(&self) -> &ProjectName {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn color(&self) -> &ProjectColor {
        &self.color
    }
}

/// Состояние проекта из хранилища: используется только для восстановления сущности.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectSnapshot {
    pub id: ProjectId,
    pub name: ProjectName,
    pub description: String,
    pub color: ProjectColor,
    pub position: i32,
    pub archived: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// Проект на доске: имя, цвет, позиция в списке и архивный флаг.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Project {
    id: ProjectId,
    name: ProjectName,
    description: String,
    color: ProjectColor,
    position: i32,
    archived: bool,
    created_at: Timestamp,
    updated_at: Timestamp,
}

impl Project {
    /// Собирает сущность из проверенного черновика; id выдаёт хранилище.
    pub fn create(id: ProjectId, draft: ProjectDraft, position: i32, now: Timestamp) -> Self {
        Self {
            id,
            name: draft.name,
            description: draft.description,
            color: draft.color,
            position,
            archived: false,
            created_at: now,
            updated_at: now,
        }
    }

    /// Восстанавливает проект из БД. Ошибка здесь означает испорченные данные, а не
    /// действие пользователя, поэтому описание проверяется повторно.
    pub fn restore(snapshot: ProjectSnapshot) -> Result<Self, DomainError> {
        Ok(Self {
            id: snapshot.id,
            name: snapshot.name,
            description: validate_description(&snapshot.description)?,
            color: snapshot.color,
            position: snapshot.position,
            archived: snapshot.archived,
            created_at: snapshot.created_at,
            updated_at: snapshot.updated_at,
        })
    }

    pub fn rename(&mut self, name: ProjectName, now: Timestamp) {
        self.name = name;
        self.updated_at = now;
    }

    pub fn describe(&mut self, description: &str, now: Timestamp) -> Result<(), DomainError> {
        self.description = validate_description(description)?;
        self.updated_at = now;
        Ok(())
    }

    pub fn recolor(&mut self, color: ProjectColor, now: Timestamp) {
        self.color = color;
        self.updated_at = now;
    }

    pub fn place_at(&mut self, position: i32, now: Timestamp) {
        self.position = position;
        self.updated_at = now;
    }

    pub fn archive(&mut self, now: Timestamp) {
        self.archived = true;
        self.updated_at = now;
    }

    pub fn unarchive(&mut self, now: Timestamp) {
        self.archived = false;
        self.updated_at = now;
    }

    pub fn id(&self) -> ProjectId {
        self.id
    }

    pub fn name(&self) -> &ProjectName {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn color(&self) -> &ProjectColor {
        &self.color
    }

    pub fn position(&self) -> i32 {
        self.position
    }

    pub fn is_archived(&self) -> bool {
        self.archived
    }

    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    pub fn updated_at(&self) -> Timestamp {
        self.updated_at
    }
}

fn validate_description(raw: &str) -> Result<String, DomainError> {
    if raw.chars().count() > MAX_PROJECT_DESCRIPTION_CHARS {
        return Err(DomainError::ProjectDescriptionTooLong {
            max: MAX_PROJECT_DESCRIPTION_CHARS,
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

    fn draft() -> ProjectDraft {
        ProjectDraft::new("Релиз 2.0", "", None).expect("черновик валиден")
    }

    fn project() -> Project {
        Project::create(ProjectId::from_raw(1), draft(), 0, now(100))
    }

    #[test]
    fn draft_trims_name_and_defaults_color() {
        let draft = ProjectDraft::new("  Релиз 2.0  ", "описание", None).expect("черновик валиден");

        assert_eq!(draft.name().as_str(), "Релиз 2.0");
        assert_eq!(draft.color().as_str(), DEFAULT_PROJECT_COLOR);
    }

    #[test]
    fn draft_normalizes_color_to_lowercase() {
        let draft = ProjectDraft::new("Проект", "", Some("#A1B2C3")).expect("цвет валиден");

        assert_eq!(draft.color().as_str(), "#a1b2c3");
    }

    #[test]
    fn blank_name_is_rejected() {
        let error = ProjectDraft::new("   ", "", None).expect_err("пустое имя недопустимо");

        assert_eq!(error, DomainError::EmptyProjectName);
    }

    #[test]
    fn name_length_counts_characters_not_bytes() {
        let ok = "я".repeat(MAX_PROJECT_NAME_CHARS);
        let too_long = "я".repeat(MAX_PROJECT_NAME_CHARS + 1);

        assert!(ProjectDraft::new(&ok, "", None).is_ok());
        assert_eq!(
            ProjectDraft::new(&too_long, "", None).expect_err("длинное имя недопустимо"),
            DomainError::ProjectNameTooLong {
                max: MAX_PROJECT_NAME_CHARS
            }
        );
    }

    #[test]
    fn description_length_is_limited() {
        let ok = "я".repeat(MAX_PROJECT_DESCRIPTION_CHARS);
        let too_long = "я".repeat(MAX_PROJECT_DESCRIPTION_CHARS + 1);

        assert!(ProjectDraft::new("Проект", &ok, None).is_ok());
        assert_eq!(
            ProjectDraft::new("Проект", &too_long, None).expect_err("длинное описание недопустимо"),
            DomainError::ProjectDescriptionTooLong {
                max: MAX_PROJECT_DESCRIPTION_CHARS
            }
        );
    }

    #[test]
    fn malformed_colors_are_rejected() {
        for raw in ["red", "#12345", "#1234567", "1234567", "#gggggg", "#12345g"] {
            assert_eq!(
                ProjectColor::parse(raw).expect_err("цвет недопустим"),
                DomainError::InvalidProjectColor(raw.to_owned()),
                "цвет {raw:?} должен быть отвергнут"
            );
        }
    }

    #[test]
    fn new_project_is_active_and_starts_at_creation_time() {
        let project = project();

        assert!(!project.is_archived());
        assert_eq!(project.created_at(), now(100));
        assert_eq!(project.updated_at(), now(100));
        assert_eq!(project.position(), 0);
    }

    #[test]
    fn rename_changes_name_and_updated_at_but_not_created_at() {
        let mut project = project();

        project.rename(
            ProjectName::parse("Новое имя").expect("имя валидно"),
            now(200),
        );

        assert_eq!(project.name().as_str(), "Новое имя");
        assert_eq!(project.created_at(), now(100));
        assert_eq!(project.updated_at(), now(200));
    }

    #[test]
    fn archive_and_unarchive_toggle_flag() {
        let mut project = project();

        project.archive(now(200));
        assert!(project.is_archived());
        assert_eq!(project.updated_at(), now(200));

        project.unarchive(now(300));
        assert!(!project.is_archived());
        assert_eq!(project.updated_at(), now(300));
    }

    #[test]
    fn describe_keeps_previous_value_when_new_one_is_invalid() {
        let mut project = project();

        let error = project
            .describe(&"я".repeat(MAX_PROJECT_DESCRIPTION_CHARS + 1), now(200))
            .expect_err("длинное описание недопустимо");

        assert!(matches!(
            error,
            DomainError::ProjectDescriptionTooLong { .. }
        ));
        assert_eq!(project.description(), "");
        assert_eq!(
            project.updated_at(),
            now(100),
            "неудачная правка не трогает метку"
        );
    }

    #[test]
    fn restore_rejects_corrupted_description() {
        let snapshot = ProjectSnapshot {
            id: ProjectId::from_raw(1),
            name: ProjectName::parse("Проект").expect("имя валидно"),
            description: "я".repeat(MAX_PROJECT_DESCRIPTION_CHARS + 1),
            color: ProjectColor::default_color(),
            position: 0,
            archived: false,
            created_at: now(100),
            updated_at: now(100),
        };

        assert!(matches!(
            Project::restore(snapshot),
            Err(DomainError::ProjectDescriptionTooLong { .. })
        ));
    }

    #[test]
    fn place_at_moves_project_in_list() {
        let mut project = project();

        project.place_at(3, now(200));

        assert_eq!(project.position(), 3);
        assert_eq!(project.updated_at(), now(200));
    }
}
