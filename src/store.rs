use crate::model::{Job, Status};

#[derive(Default)]
pub struct Store {
    jobs: Vec<Job>,
    next_id: u64,
}

impl Store {
    pub fn new() -> Self {
        Self {
            jobs: Vec::new(),
            next_id: 1,
        }
    }

    pub fn add_job(
        &mut self,
        company: String,
        role: String,
        url: Option<String>,
        status: Status,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        self.jobs.push(Job {
            id,
            company,
            role,
            url,
            status,
        });

        id
    }

    pub fn list_jobs(&self) -> Vec<Job> {
        let mut out = self.jobs.clone();
        out.sort_by_key(|j| std::cmp::Reverse(j.id)); // newest first
        out
    }
}
