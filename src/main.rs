mod cli;
mod db;
mod model;

use anyhow::{Context, Result};
use clap::Parser;
use sqlx::SqlitePool;
use std::{fs, path::Path, path::PathBuf};

#[tokio::main]
async fn main() -> Result<()> {
    let cli::Args { db_path, command } = cli::Args::parse();

    let db_path = db_path.unwrap_or_else(default_db_path);
    ensure_parent_dir_exists(&db_path)?;

    let pool = db::connect(&db_path).await?;
    db::init(&pool).await?;

    run_command(command, &pool).await
}

fn default_db_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("job-tracker")
        .join("job_tracker.db")
}

fn ensure_parent_dir_exists(path: &Path) -> Result<()> {
    let parent = path.parent().context("db path has no parent directory")?;
    fs::create_dir_all(parent)?;
    Ok(())
}

async fn run_command(command: cli::Command, pool: &SqlitePool) -> Result<()> {
    match command {
        cli::Command::Add {
            company,
            role,
            url,
            status,
        } => {
            let id = db::insert_job(pool, &company, &role, url.as_deref(), status).await?;
            println!("Added job #{id}: {company} — {role}");
        }

        cli::Command::List => {
            let jobs: Vec<crate::model::Job> = db::list_jobs(pool).await?;
            if jobs.is_empty() {
                println!("No jobs yet.");
            } else {
                for j in jobs {
                    println!(
                        "#{:03} | {} | {} | {} | {}",
                        j.id,
                        j.company,
                        j.role,
                        j.status,
                        j.url.as_deref().unwrap_or("-")
                    );
                }
            }
        }

        cli::Command::UpdateStatus { id, status } => {
            db::update_status(pool, id as i64, status).await?;
            println!("Updated job #{id} -> {status}");
        }

        cli::Command::Note { command } => match command {
            cli::NoteCommand::Add { id, text } => {
                let note_id = db::insert_note(pool, id as i64, &text).await?;
                println!("Added note #{note_id} to job #{id}");
            }

            cli::NoteCommand::List { id } => {
                let notes = db::list_notes(pool, id as i64).await?;
                if notes.is_empty() {
                    println!("No notes for job #{id}.");
                } else {
                    for n in notes {
                        println!("#{:03} | {} | {}", n.id, n.created_at, n.text);
                    }
                }
            }
        },
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        assert_eq!(2 + 2, 4);
    }
}
