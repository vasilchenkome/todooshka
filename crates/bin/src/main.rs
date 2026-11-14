//! Composition root: конфигурация, сборка зависимостей, CLI и запуск сервера.

mod cli;
mod clock;
mod config;
mod logging;
mod security;

use std::error::Error;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use sqlx::SqlitePool;
use todooshka_app::{
    AttachmentService, AuthService, ChangePassword, Clock, CommentService, CreateUser,
    ProjectRepository, ProjectService, SubtaskService, TagService, TaskService, TaskStore,
};
use todooshka_http::{AppState, ReadinessProbe, auth::LoginLimiter};
use todooshka_sqlite::{
    SqliteAttachments, SqliteComments, SqliteProjects, SqliteSessions, SqliteSubtasks, SqliteTags,
    SqliteTasks, SqliteUsers,
};

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
        cli::Command::Healthcheck => report(healthcheck()),
        cli::Command::Backup { target } => report(backup_database(&target).await),
    }
}

async fn backup_database(target: &str) -> Result<(), Box<dyn Error>> {
    let pool = open_database().await?;
    let path = std::path::Path::new(target);
    todooshka_sqlite::backup_to(&pool, path).await?;
    let size = std::fs::metadata(path)?.len();
    println!("копия базы: {} ({} КБ)", path.display(), size / 1024);
    pool.close().await;
    Ok(())
}

/// Проверка для контейнера: читает readiness-эндпоинт по тому же адресу, что и сервер.
fn healthcheck() -> Result<(), Box<dyn Error>> {
    let bind = config::bind_addr()?;
    let port = bind.port();
    let target = SocketAddr::from(([127, 0, 0, 1], port));

    let mut stream = TcpStream::connect_timeout(&target, Duration::from_secs(2))?;
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;

    write!(
        stream,
        "GET /api/health/ready HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"
    )?;

    let mut response = String::new();
    stream.read_to_string(&mut response)?;

    if response.starts_with("HTTP/1.1 200") {
        return Ok(());
    }

    Err(format!(
        "healthcheck ответил: {}",
        response.lines().next().unwrap_or("пусто")
    )
    .into())
}

struct DatabaseReadiness {
    pool: SqlitePool,
}

#[async_trait]
impl ReadinessProbe for DatabaseReadiness {
    async fn check(&self) -> bool {
        sqlx::query("SELECT 1").fetch_one(&self.pool).await.is_ok()
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
        Arc::clone(&clock),
    ));
    let subtasks = Arc::new(SubtaskService::new(
        Arc::new(SqliteSubtasks::new(pool.clone())),
        Arc::clone(&clock),
    ));
    let tags = Arc::new(TagService::new(Arc::new(SqliteTags::new(pool.clone()))));
    let comments = Arc::new(CommentService::new(
        Arc::new(SqliteComments::new(pool.clone())),
        Arc::clone(&clock),
    ));
    let attachments = Arc::new(AttachmentService::new(
        Arc::new(SqliteAttachments::new(pool.clone())),
        Arc::clone(&clock),
    ));
    let auth = Arc::new(AuthService::new(
        Arc::new(SqliteUsers::new(pool.clone())),
        Arc::new(SqliteSessions::new(pool.clone())),
        Arc::new(Argon2PasswordHasher),
        Arc::new(RandomSessionTokens),
        clock,
    ));

    let state = AppState {
        projects,
        tasks,
        subtasks,
        tags,
        comments,
        attachments,
        auth,
        login_limiter: LoginLimiter::default(),
        cookie_secure: config::cookie_secure(),
        readiness: Arc::new(DatabaseReadiness { pool: pool.clone() }),
    };

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
    let ctrl_c = async {
        if let Err(err) = tokio::signal::ctrl_c().await {
            tracing::error!(%err, "не удалось подписаться на Ctrl+C");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(err) => tracing::error!(%err, "не удалось подписаться на SIGTERM"),
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {}
        _ = terminate => {}
    }

    tracing::info!("получен сигнал остановки, завершаю работу");

    tokio::spawn(async {
        tokio::time::sleep(Duration::from_secs(10)).await;
        tracing::warn!("соединения не закрылись за 10 секунд, завершаюсь принудительно");
        std::process::exit(0);
    });
}
