use crate::model::{Job, Status};
use anyhow::{Context, Result};
use sqlx::{Row, SqlitePool, sqlite::SqliteConnectOptions};
use std::{path::Path, str::FromStr};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("job not found: {0}")]
    NotFound(i64),
}

pub async fn connect(db_path: &Path) -> Result<SqlitePool> {
    let path_str = db_path.to_str().context("db path contains invalid UTF-8")?;

    let url = format!("sqlite:{path_str}");
    let opts = SqliteConnectOptions::from_str(&url)?.create_if_missing(true);

    Ok(SqlitePool::connect_with(opts).await?)
}

pub async fn init(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS jobs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            company TEXT NOT NULL,
            role TEXT NOT NULL,
            url TEXT,
            status TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

fn status_to_str(status: Status) -> &'static str {
    match status {
        Status::Applied => "applied",
        Status::Interviewing => "interviewing",
        Status::Offer => "offer",
        Status::Rejected => "rejected",
    }
}

fn str_to_status(s: &str) -> Status {
    match s {
        "interviewing" => Status::Interviewing,
        "offer" => Status::Offer,
        "rejected" => Status::Rejected,
        _ => Status::Applied,
    }
}

pub async fn insert_job(
    pool: &SqlitePool,
    company: &str,
    role: &str,
    url: Option<&str>,
    status: Status,
) -> Result<i64> {
    let result = sqlx::query(
        r#"
        INSERT INTO jobs (company, role, url, status)
        VALUES (?1, ?2, ?3, ?4);
        "#,
    )
    .bind(company)
    .bind(role)
    .bind(url)
    .bind(status_to_str(status))
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

pub async fn list_jobs(pool: &SqlitePool) -> Result<Vec<Job>> {
    let rows = sqlx::query(
        r#"
        SELECT id, company, role, url, status
        FROM jobs
        ORDER BY id DESC;
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| Job {
            id: r.get::<i64, _>("id") as u64,
            company: r.get::<String, _>("company"),
            role: r.get::<String, _>("role"),
            url: r.get::<Option<String>, _>("url"),
            status: str_to_status(&r.get::<String, _>("status")),
        })
        .collect())
}

pub async fn update_status(pool: &SqlitePool, id: i64, status: Status) -> Result<()> {
    let result = sqlx::query(
        r#"
        UPDATE jobs
        SET status = ?1
        WHERE id = ?2;
        "#,
    )
    .bind(status_to_str(status))
    .bind(id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(DbError::NotFound(id).into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn insert_and_list_roundtrip() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");

        let pool = connect(&db_path).await.unwrap();
        init(&pool).await.unwrap();

        let id = insert_job(
            &pool,
            "Acme",
            "Backend Engineer",
            Some("https://example.com"),
            Status::Applied,
        )
        .await
        .unwrap();

        let jobs = list_jobs(&pool).await.unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].id, id as u64);
        assert_eq!(jobs[0].company, "Acme");
        assert_eq!(jobs[0].status, Status::Applied);
    }

    #[tokio::test]
    async fn update_status_errors_when_missing() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");

        let pool = connect(&db_path).await.unwrap();
        init(&pool).await.unwrap();

        let err = update_status(&pool, 999, Status::Interviewing)
            .await
            .unwrap_err();

        assert!(err.to_string().contains("job not found"));
    }
}
