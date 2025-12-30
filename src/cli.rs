use crate::model::Status;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "job")]
#[command(about = "Track job applications locally", long_about = None)]
pub struct Args {
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
