use crate::model::{Job, Status};
use anyhow::{Context, Result};
use sqlx::{Row, SqlitePool, sqlite::SqliteConnectOptions};
use std::{path::Path, str::FromStr};

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
