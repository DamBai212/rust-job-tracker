use crate::model::Status;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "job")]
#[command(about = "Track job applications locally", long_about = None)]
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
}
