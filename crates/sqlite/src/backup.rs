//! Копия базы для бэкапа: `VACUUM INTO` и проверка целостности копии.

use std::path::Path;

use sqlx::SqlitePool;

use crate::error::StorageError;

/// Снимает согласованную копию базы в новый файл и проверяет её `integrity_check`.
///
/// `VACUUM INTO` не перезаписывает существующий файл: старые копии не затираются молча.
pub async fn backup_to(pool: &SqlitePool, target: &Path) -> Result<(), StorageError> {
    if target.exists() {
        return Err(StorageError::Backup(format!(
            "файл {} уже существует",
            target.display()
        )));
    }

    if let Some(parent) = target.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent).map_err(|error| {
            StorageError::Backup(format!("папка {} не создана: {error}", parent.display()))
        })?;
    }

    sqlx::query("VACUUM INTO ?1")
        .bind(target.to_string_lossy().to_string())
        .execute(pool)
        .await
        .map_err(|error| StorageError::Backup(format!("VACUUM INTO не выполнен: {error}")))?;

    let url = format!("sqlite://{}", target.to_string_lossy().replace('\\', "/"));
    let copy = crate::connect(&url).await?;
    let integrity: (String,) = sqlx::query_as("PRAGMA integrity_check")
        .fetch_one(&copy)
        .await
        .map_err(|error| StorageError::Backup(format!("integrity_check не выполнен: {error}")))?;
    copy.close().await;

    if integrity.0 != "ok" {
        let _ = std::fs::remove_file(target);
        return Err(StorageError::Backup(format!(
            "копия повреждена: {}",
            integrity.0
        )));
    }

    Ok(())
}
