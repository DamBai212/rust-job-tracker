use crate::model::{Job, Note, Status};
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
    sqlx::query("PRAGMA foreign_keys = ON;")
        .execute(pool)
        .await?;

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

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS notes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            job_id INTEGER NOT NULL,
            text TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY(job_id) REFERENCES jobs(id) ON DELETE CASCADE
        );
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/* ---------- helpers ---------- */

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

async fn job_exists(pool: &SqlitePool, job_id: i64) -> Result<bool> {
    let row = sqlx::query("SELECT 1 FROM jobs WHERE id = ?1 LIMIT 1;")
        .bind(job_id)
        .fetch_optional(pool)
        .await?;

    Ok(row.is_some())
}

/* ---------- jobs ---------- */

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

pub async fn delete_job(pool: &SqlitePool, id: i64) -> Result<()> {
    let result = sqlx::query(
        r#"
        DELETE FROM jobs
        WHERE id = ?1;
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(DbError::NotFound(id).into());
    }

    Ok(())
}

/* ---------- notes ---------- */

pub async fn insert_note(pool: &SqlitePool, job_id: i64, text: &str) -> Result<i64> {
    if !job_exists(pool, job_id).await? {
        return Err(DbError::NotFound(job_id).into());
    }

    let result = sqlx::query(
        r#"
        INSERT INTO notes (job_id, text)
        VALUES (?1, ?2);
        "#,
    )
    .bind(job_id)
    .bind(text)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

pub async fn list_notes(pool: &SqlitePool, job_id: i64) -> Result<Vec<Note>> {
    if !job_exists(pool, job_id).await? {
        return Err(DbError::NotFound(job_id).into());
    }

    let rows = sqlx::query(
        r#"
        SELECT id, job_id, text, created_at
        FROM notes
        WHERE job_id = ?1
        ORDER BY id DESC;
        "#,
    )
    .bind(job_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| Note {
            id: r.get::<i64, _>("id") as u64,
            job_id: r.get::<i64, _>("job_id") as u64,
            text: r.get::<String, _>("text"),
            created_at: r.get::<String, _>("created_at"),
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn delete_job_cascades_notes() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");

        let pool = connect(&db_path).await.unwrap();
        init(&pool).await.unwrap();

        let job_id = insert_job(&pool, "Acme", "Backend", None, Status::Applied)
            .await
            .unwrap();

        insert_note(&pool, job_id, "note 1").await.unwrap();
        insert_note(&pool, job_id, "note 2").await.unwrap();

        delete_job(&pool, job_id).await.unwrap();

        let remaining = sqlx::query("SELECT COUNT(*) as c FROM notes WHERE job_id = ?1;")
            .bind(job_id)
            .fetch_one(&pool)
            .await
            .unwrap()
            .get::<i64, _>("c");

        assert_eq!(remaining, 0);
    }
}
