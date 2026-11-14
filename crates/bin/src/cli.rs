//! Разбор аргументов командной строки без внешних парсеров: команд мало и они плоские.

use std::ffi::OsString;

pub const USAGE: &str = "\
Todooshka — Kanban по рабочим проектам.

Использование:
  todooshka [serve]                       запустить сервер
  todooshka user add -u <логин> [--password-stdin]     создать пользователя
  todooshka user passwd -u <логин> [--password-stdin]  сменить пароль (отзывает сессии)
  todooshka user list                     список пользователей
  todooshka healthcheck                   проверить готовность запущенного сервера
  todooshka backup <файл>                 снять копию базы и проверить её целостность

Флаги:
  --password-stdin   читать пароль из stdin вместо скрытого ввода
  -h, --help         эта справка
";

/// Что просили сделать при запуске.
#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Serve,
    UserAdd {
        username: String,
        password_from_stdin: bool,
    },
    UserPasswd {
        username: String,
        password_from_stdin: bool,
    },
    UserList,
    Healthcheck,
    Backup {
        target: String,
    },
    Help,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum CliError {
    #[error("неизвестная команда: {0}\n\n{USAGE}")]
    Unknown(String),

    #[error("не хватает логина: укажите -u <логин> для команды {0}")]
    MissingUsername(&'static str),

    #[error("у флага {0} нет значения")]
    MissingValue(&'static str),

    #[error("укажите путь к файлу копии: todooshka backup <файл>")]
    MissingBackupPath,
}

/// Разбирает аргументы без имени программы. `--help` и `help` печатают справку.
pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Command, CliError> {
    let args = args
        .into_iter()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect::<Vec<_>>();

    match args.as_slice() {
        [] => Ok(Command::Serve),
        [command] if command == "serve" => Ok(Command::Serve),
        [command] if command == "healthcheck" => Ok(Command::Healthcheck),
        [command, target] if command == "backup" => Ok(Command::Backup {
            target: target.clone(),
        }),
        [command] if command == "backup" => Err(CliError::MissingBackupPath),
        [command] if command == "-h" || command == "--help" || command == "help" => {
            Ok(Command::Help)
        }
        [user, action, rest @ ..] if user == "user" => match action.as_str() {
            "list" if rest.is_empty() => Ok(Command::UserList),
            "add" => user_command(rest, "add", |username, password_from_stdin| {
                Command::UserAdd {
                    username,
                    password_from_stdin,
                }
            }),
            "passwd" => user_command(rest, "passwd", |username, password_from_stdin| {
                Command::UserPasswd {
                    username,
                    password_from_stdin,
                }
            }),
            other => Err(CliError::Unknown(format!("user {other}"))),
        },
        [other, ..] => Err(CliError::Unknown(other.clone())),
    }
}

fn user_command(
    rest: &[String],
    name: &'static str,
    build: impl FnOnce(String, bool) -> Command,
) -> Result<Command, CliError> {
    let mut username = None;
    let mut password_from_stdin = false;
    let mut index = 0;

    while index < rest.len() {
        match rest[index].as_str() {
            "-u" | "--username" => {
                let value = rest
                    .get(index + 1)
                    .ok_or(CliError::MissingValue("--username"))?;
                username = Some(value.clone());
                index += 2;
            }
            "--password-stdin" => {
                password_from_stdin = true;
                index += 1;
            }
            _ => index += 1,
        }
    }

    let username = username.ok_or(CliError::MissingUsername(name))?;
    Ok(build(username, password_from_stdin))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_args(args: &[&str]) -> Result<Command, CliError> {
        parse(args.iter().map(OsString::from))
    }

    #[test]
    fn empty_args_start_the_server() {
        assert_eq!(parse_args(&[]).expect("команда"), Command::Serve);
        assert_eq!(parse_args(&["serve"]).expect("команда"), Command::Serve);
    }

    #[test]
    fn user_add_reads_username_and_flag() {
        assert_eq!(
            parse_args(&["user", "add", "-u", "kirill"]).expect("команда"),
            Command::UserAdd {
                username: "kirill".to_owned(),
                password_from_stdin: false
            }
        );
        assert_eq!(
            parse_args(&["user", "add", "--username", "kirill", "--password-stdin"])
                .expect("команда"),
            Command::UserAdd {
                username: "kirill".to_owned(),
                password_from_stdin: true
            }
        );
    }

    #[test]
    fn user_passwd_and_list_are_recognized() {
        assert_eq!(
            parse_args(&["user", "passwd", "-u", "kirill"]).expect("команда"),
            Command::UserPasswd {
                username: "kirill".to_owned(),
                password_from_stdin: false
            }
        );
        assert_eq!(
            parse_args(&["user", "list"]).expect("команда"),
            Command::UserList
        );
    }

    #[test]
    fn healthcheck_is_recognized() {
        assert_eq!(
            parse_args(&["healthcheck"]).expect("команда"),
            Command::Healthcheck
        );
    }

    #[test]
    fn backup_takes_a_path() {
        assert_eq!(
            parse_args(&["backup", "copy.db"]).expect("команда"),
            Command::Backup {
                target: "copy.db".to_owned()
            }
        );
        assert_eq!(
            parse_args(&["backup"]).expect_err("нужен путь"),
            CliError::MissingBackupPath
        );
    }

    #[test]
    fn help_is_recognized() {
        for args in [&["-h"][..], &["--help"], &["help"]] {
            assert_eq!(parse_args(args).expect("команда"), Command::Help);
        }
    }

    #[test]
    fn missing_username_is_an_error() {
        assert_eq!(
            parse_args(&["user", "add"]).expect_err("нужен логин"),
            CliError::MissingUsername("add")
        );
        assert_eq!(
            parse_args(&["user", "passwd", "-u"]).expect_err("нужно значение"),
            CliError::MissingValue("--username")
        );
    }

    #[test]
    fn unknown_command_is_an_error() {
        assert!(matches!(
            parse_args(&["user", "delete"]).expect_err("команда неизвестна"),
            CliError::Unknown(_)
        ));
        assert!(matches!(
            parse_args(&["deploy"]).expect_err("команда неизвестна"),
            CliError::Unknown(_)
        ));
    }
}
