use crate::error::DomainError;
use crate::time::Timestamp;

/// Ограничения учётной записи: логин короткий, пароль длинный.
pub const MAX_USERNAME_CHARS: usize = 64;
pub const MIN_PASSWORD_CHARS: usize = 10;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct UserId(i64);

impl UserId {
    pub const fn from_raw(value: i64) -> Self {
        Self(value)
    }

    pub const fn as_raw(self) -> i64 {
        self.0
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

/// Логин: обрезан по краям, не пустой, без управляющих символов.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Username(String);

impl Username {
    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(DomainError::EmptyUsername);
        }
        if trimmed.chars().count() > MAX_USERNAME_CHARS {
            return Err(DomainError::UsernameTooLong {
                max: MAX_USERNAME_CHARS,
            });
        }
        if trimmed.chars().any(char::is_control) {
            return Err(DomainError::InvalidUsername);
        }
        Ok(Self(trimmed.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Хеш пароля — непрозрачное значение: домен не знает, чем именно он посчитан.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PasswordHash(String);

impl PasswordHash {
    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        if raw.trim().is_empty() {
            return Err(DomainError::InvalidPasswordHash);
        }
        Ok(Self(raw.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Учётная запись: логин, хеш пароля и момент создания.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct User {
    id: UserId,
    username: Username,
    password_hash: PasswordHash,
    created_at: Timestamp,
}

impl User {
    pub fn new(
        id: UserId,
        username: Username,
        password_hash: PasswordHash,
        created_at: Timestamp,
    ) -> Self {
        Self {
            id,
            username,
            password_hash,
            created_at,
        }
    }

    pub fn id(&self) -> UserId {
        self.id
    }

    pub fn username(&self) -> &Username {
        &self.username
    }

    pub fn password_hash(&self) -> &PasswordHash {
        &self.password_hash
    }

    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }
}

/// Политика пароля проверяется до хеширования: минимум символов и не только пробелы.
pub fn validate_password(password: &str) -> Result<(), DomainError> {
    if password.chars().count() < MIN_PASSWORD_CHARS {
        return Err(DomainError::WeakPassword {
            min: MIN_PASSWORD_CHARS,
        });
    }
    if password.trim().is_empty() {
        return Err(DomainError::WeakPassword {
            min: MIN_PASSWORD_CHARS,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn username_is_trimmed() {
        let username = Username::parse("  kirill  ").expect("логин валиден");

        assert_eq!(username.as_str(), "kirill");
    }

    #[test]
    fn blank_username_is_rejected() {
        assert_eq!(
            Username::parse("   ").expect_err("пустой логин недопустим"),
            DomainError::EmptyUsername
        );
    }

    #[test]
    fn username_length_is_limited_in_characters() {
        let ok = "я".repeat(MAX_USERNAME_CHARS);
        let too_long = "я".repeat(MAX_USERNAME_CHARS + 1);

        assert!(Username::parse(&ok).is_ok());
        assert_eq!(
            Username::parse(&too_long).expect_err("длинный логин недопустим"),
            DomainError::UsernameTooLong {
                max: MAX_USERNAME_CHARS
            }
        );
    }

    #[test]
    fn control_characters_in_username_are_rejected() {
        assert_eq!(
            Username::parse("kir\0ill").expect_err("управляющий символ недопустим"),
            DomainError::InvalidUsername
        );
    }

    #[test]
    fn password_shorter_than_limit_is_rejected() {
        let short = "a".repeat(MIN_PASSWORD_CHARS - 1);

        assert_eq!(
            validate_password(&short).expect_err("короткий пароль недопустим"),
            DomainError::WeakPassword {
                min: MIN_PASSWORD_CHARS
            }
        );
    }

    #[test]
    fn password_of_spaces_is_rejected() {
        let spaces = " ".repeat(MIN_PASSWORD_CHARS + 5);

        assert!(validate_password(&spaces).is_err());
    }

    #[test]
    fn long_enough_password_passes() {
        let password = "a".repeat(MIN_PASSWORD_CHARS);

        assert!(validate_password(&password).is_ok());
    }

    #[test]
    fn empty_password_hash_is_rejected() {
        assert_eq!(
            PasswordHash::parse("  ").expect_err("пустой хеш недопустим"),
            DomainError::InvalidPasswordHash
        );
    }
}
