//! Job queue management and batching

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationJob {
    pub id: String,
    pub prompt: String,
    pub status: JobStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobStatus {
    Queued,
    Running,
    Completed,
    Failed,
}

pub struct QueueManager {
    jobs: Vec<GenerationJob>,
}

impl QueueManager {
    pub fn new() -> Self {
        Self { jobs: Vec::new() }
    }

    pub fn add_job(&mut self, prompt: String) -> String {
        let id = Uuid::new_v4().to_string();
        let job = GenerationJob {
            id: id.clone(),
            prompt,
            status: JobStatus::Queued,
        };
        self.jobs.push(job);
        id
    }

    pub fn get_jobs(&self) -> &[GenerationJob] {
        &self.jobs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queue_manager_add_job() {
        let mut manager = QueueManager::new();
        let job_id = manager.add_job("test prompt".to_string());
        assert!(!job_id.is_empty());
        assert_eq!(manager.get_jobs().len(), 1);
    }
}
