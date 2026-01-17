use crate::model::Status;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "job")]
#[command(about = "Track job applications locally")]
pub struct Args {
    /// Override database path (for testing or custom setups)
    #[arg(long)]
    pub db_path: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Add {
        #[arg(long)]
        company: String,
        #[arg(long)]
        role: String,
        #[arg(long)]
        url: Option<String>,
        #[arg(long, default_value = "applied")]
        status: Status,
    },
    List,
    UpdateStatus {
        #[arg(long)]
        id: u64,
        #[arg(long)]
        status: Status,
    },
    Delete {
        #[arg(long)]
        id: u64,
    },
    Note {
        #[command(subcommand)]
        command: NoteCommand,
    },
}

#[derive(Subcommand, Debug)]
pub enum NoteCommand {
    Add {
        #[arg(long)]
        id: u64,
        #[arg(long)]
        text: String,
    },
    List {
        #[arg(long)]
        id: u64,
    },
}
