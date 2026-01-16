//! Generation queue management with async execution

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationParams {
    pub prompt: String,
    pub negative_prompt: Option<String>,
    pub steps: u32,
    pub cfg_scale: f64,
    pub width: u32,
    pub height: u32,
    pub seed: i64,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationJob {
    pub id: String,
    pub params: GenerationParams,
    pub status: JobStatus,
    pub progress: f32,
    pub created_at: i64,
    pub started_at: Option<i64>,
    pub completed_at: Option<i64>,
    pub result_path: Option<String>,
    pub error: Option<String>,
}

impl GenerationJob {
    pub fn new(params: GenerationParams) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            params,
            status: JobStatus::Pending,
            progress: 0.0,
            created_at: chrono::Utc::now().timestamp(),
            started_at: None,
            completed_at: None,
            result_path: None,
            error: None,
        }
    }
}

pub struct QueueManager {
    jobs: Arc<RwLock<Vec<GenerationJob>>>,
    running: Arc<Mutex<usize>>,
    max_concurrent: usize,
}

impl QueueManager {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            jobs: Arc::new(RwLock::new(Vec::new())),
            running: Arc::new(Mutex::new(0)),
            max_concurrent,
        }
    }

    pub async fn add_job(&self, params: GenerationParams) -> String {
        let job = GenerationJob::new(params);
        let job_id = job.id.clone();

        let mut jobs = self.jobs.write().await;
        jobs.push(job);

        job_id
    }

    pub async fn get_jobs(&self) -> Vec<GenerationJob> {
        self.jobs.read().await.clone()
    }

    pub async fn get_job(&self, job_id: &str) -> Option<GenerationJob> {
        self.jobs
            .read()
            .await
            .iter()
            .find(|j| j.id == job_id)
            .cloned()
    }

    pub async fn update_job_status(&self, job_id: &str, status: JobStatus) {
        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
            job.status = status.clone();
            match status {
                JobStatus::Running => {
                    job.started_at = Some(chrono::Utc::now().timestamp());
                }
                JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled => {
                    job.completed_at = Some(chrono::Utc::now().timestamp());
                }
                _ => {}
            }
        }
    }

    pub async fn update_job_progress(&self, job_id: &str, progress: f32) {
        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
            job.progress = progress;
        }
    }

    pub async fn complete_job(&self, job_id: &str, result_path: String) {
        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
            job.status = JobStatus::Completed;
            job.progress = 1.0;
            job.result_path = Some(result_path);
            job.completed_at = Some(chrono::Utc::now().timestamp());
        }
    }

    pub async fn fail_job(&self, job_id: &str, error: String) {
        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
            job.status = JobStatus::Failed;
            job.error = Some(error);
            job.completed_at = Some(chrono::Utc::now().timestamp());
        }
    }

    pub async fn cancel_job(&self, job_id: &str) -> bool {
        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
            if job.status == JobStatus::Pending {
                job.status = JobStatus::Cancelled;
                job.completed_at = Some(chrono::Utc::now().timestamp());
                return true;
            }
        }
        false
    }

    pub async fn can_start_job(&self) -> bool {
        let running = *self.running.lock().await;
        running < self.max_concurrent
    }

    pub async fn increment_running(&self) {
        let mut running = self.running.lock().await;
        *running += 1;
    }

    pub async fn decrement_running(&self) {
        let mut running = self.running.lock().await;
        if *running > 0 {
            *running -= 1;
        }
    }

    pub async fn clear_completed(&self) {
        let mut jobs = self.jobs.write().await;
        jobs.retain(|j| {
            !matches!(
                j.status,
                JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled
            )
        });
    }
}
