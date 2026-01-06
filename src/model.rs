use clap::ValueEnum;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Status {
    Applied,
    Interviewing,
    Offer,
    Rejected,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Status::Applied => "applied",
            Status::Interviewing => "interviewing",
            Status::Offer => "offer",
            Status::Rejected => "rejected",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
    pub id: u64,
    pub company: String,
    pub role: String,
    pub url: Option<String>,
    pub status: Status,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    pub id: u64,
    pub job_id: u64,
    pub text: String,
    pub created_at: String,
}