use crate::error::DomainError;
use crate::task::TaskId;
use crate::time::Timestamp;

/// Лимит одного файла: больше в личном трекере не прикрепляют.
pub const MAX_ATTACHMENT_BYTES: usize = 10 * 1024 * 1024;
/// Лимит файлов на задачу.
pub const MAX_ATTACHMENTS_PER_TASK: usize = 10;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AttachmentId(i64);

impl AttachmentId {
    pub const fn from_raw(value: i64) -> Self {
        Self(value)
    }

    pub const fn as_raw(self) -> i64 {
        self.0
    }
}

impl std::fmt::Display for AttachmentId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

/// Имя файла без пути: последний сегмент, обрезанный, до 255 символов.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttachmentFilename(String);

impl AttachmentFilename {
    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        let name = raw
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or(raw)
            .trim()
            .trim_matches('"');

        if name.is_empty() {
            return Err(DomainError::EmptyAttachmentFilename);
        }
        if name.chars().count() > 255 {
            return Err(DomainError::AttachmentFilenameTooLong);
        }
        if name.chars().any(char::is_control) {
            return Err(DomainError::InvalidAttachmentFilename);
        }

        Ok(Self(name.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Тип файла определяется расширением: содержимому от клиента не доверяем.
pub fn content_type_for(filename: &str) -> &'static str {
    let extension = filename
        .rsplit('.')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();

    match extension.as_str() {
        "pdf" => "application/pdf",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "txt" => "text/plain; charset=utf-8",
        "md" => "text/markdown; charset=utf-8",
        "csv" => "text/csv; charset=utf-8",
        "json" => "application/json",
        "zip" => "application/zip",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xls" => "application/vnd.ms-excel",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        _ => "application/octet-stream",
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AttachmentMeta {
    pub id: AttachmentId,
    pub task_id: TaskId,
    pub filename: AttachmentFilename,
    pub content_type: String,
    pub size: i64,
    pub created_at: Timestamp,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filename_drops_path_and_quotes() {
        assert_eq!(
            AttachmentFilename::parse("C:\\\\docs\\\\  \"план.pdf\" ")
                .expect("имя")
                .as_str(),
            "план.pdf"
        );
        assert_eq!(
            AttachmentFilename::parse("../../etc/passwd")
                .expect("имя")
                .as_str(),
            "passwd"
        );
    }

    #[test]
    fn empty_and_long_names_are_rejected() {
        assert_eq!(
            AttachmentFilename::parse("   ").expect_err("пусто"),
            DomainError::EmptyAttachmentFilename
        );
        let long = format!("{}.pdf", "я".repeat(260));
        assert_eq!(
            AttachmentFilename::parse(&long).expect_err("длинно"),
            DomainError::AttachmentFilenameTooLong
        );
    }

    #[test]
    fn content_type_comes_from_extension() {
        assert_eq!(content_type_for("план.PDF"), "application/pdf");
        assert_eq!(content_type_for("notes.md"), "text/markdown; charset=utf-8");
        assert_eq!(
            content_type_for("archive.tar.gz"),
            "application/octet-stream"
        );
    }
}
