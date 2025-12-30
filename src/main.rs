mod cli;
mod model;
mod store;

use clap::Parser;

fn main() {
    let args = cli::Args::parse();
    let mut store = store::Store::new();

    match args.command {
        cli::Command::Add {
            company,
            role,
            url,
            status,
        } => {
            let id = store.add_job(company, role, url, status);
            println!("Added job #{id}");
        }
        cli::Command::List => {
            let jobs = store.list_jobs();
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Status;

    #[test]
    fn smoke_still_passes() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn add_job_increments_ids() {
        let mut s = store::Store::new();
        let a = s.add_job("Acme".into(), "Backend".into(), None, Status::Applied);
        let b = s.add_job("Beta".into(), "Rust Dev".into(), None, Status::Interviewing);
        assert_eq!(a, 1);
        assert_eq!(b, 2);
    }

    #[test]
    fn list_jobs_newest_first() {
        let mut s = store::Store::new();
        s.add_job("A".into(), "R1".into(), None, Status::Applied);
        s.add_job("B".into(), "R2".into(), None, Status::Applied);
        let jobs = s.list_jobs();
        assert_eq!(jobs[0].company, "B");
        assert_eq!(jobs[1].company, "A");
    }
}
