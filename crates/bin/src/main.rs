//! Composition root: конфигурация, сборка зависимостей, CLI и запуск сервера.

mod cli;
mod clock;
mod config;
mod logging;
mod security;

use std::error::Error;
use std::process::ExitCode;
use std::sync::Arc;

use sqlx::SqlitePool;
use todooshka_app::{
    AuthService, ChangePassword, Clock, CreateUser, ProjectRepository, ProjectService, TaskService,
    TaskStore,
};
use todooshka_http::AppState;
use todooshka_sqlite::{SqliteProjects, SqliteSessions, SqliteTasks, SqliteUsers};

use crate::clock::SystemClock;
use crate::security::{Argon2PasswordHasher, RandomSessionTokens};

#[tokio::main]
async fn main() -> ExitCode {
    logging::init();

    let command = match cli::parse(std::env::args_os().skip(1)) {
        Ok(command) => command,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::from(2);
        }
    };

    match command {
        cli::Command::Help => {
            println!("{}", cli::USAGE);
            ExitCode::SUCCESS
        }
        cli::Command::Serve => report(run_server().await),
        cli::Command::UserAdd {
            username,
            password_from_stdin,
        } => report(add_user(&username, password_from_stdin).await),
        cli::Command::UserPasswd {
            username,
            password_from_stdin,
        } => report(change_password(&username, password_from_stdin).await),
        cli::Command::UserList => report(list_users().await),
    }
}

fn report(result: Result<(), Box<dyn Error>>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            tracing::error!("{error}");
            eprintln!("ошибка: {error}");
            ExitCode::FAILURE
        }
    }
}

async fn run_server() -> Result<(), Box<dyn Error>> {
    let pool = open_database().await?;
    tracing::info!("миграции применены");

    let projects_repository: Arc<dyn ProjectRepository> =
        Arc::new(SqliteProjects::new(pool.clone()));
    let tasks_repository: Arc<dyn TaskStore> = Arc::new(SqliteTasks::new(pool.clone()));
    let clock: Arc<dyn Clock> = Arc::new(SystemClock);

    let projects = Arc::new(ProjectService::new(
        Arc::clone(&projects_repository),
        Arc::clone(&clock),
    ));
    let tasks = Arc::new(TaskService::new(
        tasks_repository,
        projects_repository,
        clock,
    ));

    let state = AppState { projects, tasks };

    let addr = config::bind_addr()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;

    tracing::info!(%addr, version = env!("CARGO_PKG_VERSION"), "todooshka запущен");

    axum::serve(listener, todooshka_http::router(state))
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    pool.close().await;
    Ok(())
}

async fn add_user(username: &str, password_from_stdin: bool) -> Result<(), Box<dyn Error>> {
    let password = read_password(password_from_stdin)?;
    let pool = open_database().await?;
    let auth = auth_service(&pool);

    let user = auth
        .create_user(CreateUser {
            username: username.to_owned(),
            password,
        })
        .await?;

    println!(
        "создан пользователь {} (id {})",
        user.username().as_str(),
        user.id()
    );
    pool.close().await;
    Ok(())
}

async fn change_password(username: &str, password_from_stdin: bool) -> Result<(), Box<dyn Error>> {
    let password = read_password(password_from_stdin)?;
    let pool = open_database().await?;
    let auth = auth_service(&pool);

    auth.change_password(ChangePassword {
        username: username.to_owned(),
        password,
    })
    .await?;

    println!("пароль пользователя {username} изменён, активные сессии отозваны");
    pool.close().await;
    Ok(())
}

async fn list_users() -> Result<(), Box<dyn Error>> {
    let pool = open_database().await?;
    let auth = auth_service(&pool);
    let users = auth.list_users().await?;

    if users.is_empty() {
        println!("пользователей нет");
    }
    for user in users {
        println!(
            "{}\t{}\tсоздан {}",
            user.id(),
            user.username().as_str(),
            user.created_at().as_unix_seconds()
        );
    }

    pool.close().await;
    Ok(())
}

/// Пароль вводится скрыто и дважды; для скриптов и контейнера — одна строка из stdin.
fn read_password(from_stdin: bool) -> Result<String, Box<dyn Error>> {
    if from_stdin {
        let mut line = String::new();
        std::io::stdin().read_line(&mut line)?;
        let password = line.trim_end_matches(['\r', '\n']).to_owned();
        if password.is_empty() {
            return Err("пустой пароль в stdin".into());
        }
        return Ok(password);
    }

    let first = rpassword::prompt_password("Пароль: ")?;
    let second = rpassword::prompt_password("Повторите пароль: ")?;
    if first != second {
        return Err("пароли не совпадают".into());
    }
    Ok(first)
}

fn auth_service(pool: &SqlitePool) -> AuthService {
    AuthService::new(
        Arc::new(SqliteUsers::new(pool.clone())),
        Arc::new(SqliteSessions::new(pool.clone())),
        Arc::new(Argon2PasswordHasher),
        Arc::new(RandomSessionTokens),
        Arc::new(SystemClock),
    )
}

async fn open_database() -> Result<SqlitePool, Box<dyn Error>> {
    let database_url = config::database_url();
    let pool = todooshka_sqlite::connect(&database_url).await?;
    todooshka_sqlite::migrate(&pool).await?;
    Ok(pool)
}

async fn shutdown_signal() {
    if let Err(err) = tokio::signal::ctrl_c().await {
        tracing::error!(%err, "не удалось подписаться на Ctrl+C");
        return;
    }
    tracing::info!("получен сигнал остановки, завершаю работу");
}
