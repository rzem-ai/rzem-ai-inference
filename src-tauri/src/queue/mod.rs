//! Generation queue management with async execution

mod processor;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

use crate::inference::samplers::{SamplerType, SchedulerType};
use crate::models::LoraConfig;

pub use processor::QueueProcessor;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
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
    /// Model type (schnell, dev, zimage-turbo) - optional, inferred from components
    #[serde(default)]
    pub model_type: Option<String>,
    /// Sampling algorithm (euler, euler_a, dpm_pp_2m)
    #[serde(default)]
    pub sampler: Option<SamplerType>,
    /// Noise schedule (normal, karras, exponential)
    #[serde(default)]
    pub scheduler: Option<SchedulerType>,
    /// LoRA adapters to apply with their strengths
    #[serde(default)]
    pub loras: Vec<LoraConfig>,
    /// Bundle ID - if set, use all components from bundle
    #[serde(default)]
    pub bundle_id: Option<String>,
    /// Individual component IDs (required if bundle_id not set)
    pub model_component_id: String,
    pub t5_component_id: String,
    pub clip_component_id: String,
    pub vae_component_id: String,
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
    /// Links this job to the database image entry created before processing
    pub image_id: Option<String>,
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
            image_id: None,
        }
    }

    pub fn with_image_id(mut self, image_id: String) -> Self {
        self.image_id = Some(image_id);
        self
    }
}

pub struct QueueManager {
    jobs: Arc<RwLock<HashMap<String, GenerationJob>>>,
    job_order: Arc<RwLock<Vec<String>>>,
    running: Arc<Mutex<usize>>,
    max_concurrent: usize,
}

impl QueueManager {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            jobs: Arc::new(RwLock::new(HashMap::new())),
            job_order: Arc::new(RwLock::new(Vec::new())),
            running: Arc::new(Mutex::new(0)),
            max_concurrent,
        }
    }

    pub async fn add_job(&self, params: GenerationParams) -> String {
        let job = GenerationJob::new(params);
        let job_id = job.id.clone();

        let mut jobs = self.jobs.write().await;
        let mut order = self.job_order.write().await;

        jobs.insert(job_id.clone(), job);
        order.push(job_id.clone());

        job_id
    }

    pub async fn add_job_with_image_id(&self, params: GenerationParams, image_id: String) -> String {
        let job = GenerationJob::new(params).with_image_id(image_id);
        let job_id = job.id.clone();

        let mut jobs = self.jobs.write().await;
        let mut order = self.job_order.write().await;

        jobs.insert(job_id.clone(), job);
        order.push(job_id.clone());

        job_id
    }

    pub async fn get_jobs(&self) -> Vec<GenerationJob> {
        let jobs = self.jobs.read().await;
        let order = self.job_order.read().await;

        order
            .iter()
            .filter_map(|id| jobs.get(id).cloned())
            .collect()
    }

    pub async fn get_job(&self, job_id: &str) -> Option<GenerationJob> {
        self.jobs.read().await.get(job_id).cloned()
    }

    pub async fn update_job_status(&self, job_id: &str, status: JobStatus) -> Result<(), String> {
        let mut jobs = self.jobs.write().await;

        let job = jobs
            .get_mut(job_id)
            .ok_or_else(|| format!("Job not found: {}", job_id))?;

        job.status = status;

        match status {
            JobStatus::Running => {
                job.started_at = Some(chrono::Utc::now().timestamp());
            }
            JobStatus::Completed => {
                job.progress = 1.0;
                job.completed_at = Some(chrono::Utc::now().timestamp());
            }
            JobStatus::Failed | JobStatus::Cancelled => {
                job.completed_at = Some(chrono::Utc::now().timestamp());
            }
            _ => {}
        }

        Ok(())
    }

    pub async fn update_job_progress(&self, job_id: &str, progress: f32) -> Result<(), String> {
        let mut jobs = self.jobs.write().await;

        let job = jobs
            .get_mut(job_id)
            .ok_or_else(|| format!("Job not found: {}", job_id))?;

        job.progress = progress;
        Ok(())
    }

    pub async fn complete_job(&self, job_id: &str, result_path: String) -> Result<(), String> {
        let mut jobs = self.jobs.write().await;

        let job = jobs
            .get_mut(job_id)
            .ok_or_else(|| format!("Job not found: {}", job_id))?;

        job.status = JobStatus::Completed;
        job.progress = 1.0;
        job.result_path = Some(result_path);
        job.completed_at = Some(chrono::Utc::now().timestamp());

        Ok(())
    }

    pub async fn fail_job(&self, job_id: &str, error: String) -> Result<(), String> {
        let mut jobs = self.jobs.write().await;

        let job = jobs
            .get_mut(job_id)
            .ok_or_else(|| format!("Job not found: {}", job_id))?;

        job.status = JobStatus::Failed;
        job.error = Some(error);
        job.completed_at = Some(chrono::Utc::now().timestamp());

        Ok(())
    }

    pub async fn cancel_job(&self, job_id: &str) -> bool {
        let mut jobs = self.jobs.write().await;

        if let Some(job) = jobs.get_mut(job_id) {
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
        let mut order = self.job_order.write().await;

        order.retain(|id| {
            if let Some(job) = jobs.get(id) {
                !matches!(
                    job.status,
                    JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled
                )
            } else {
                false
            }
        });

        jobs.retain(|_id, job| {
            !matches!(
                job.status,
                JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled
            )
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_params() -> GenerationParams {
        GenerationParams {
            prompt: "test prompt".to_string(),
            negative_prompt: Some("negative".to_string()),
            steps: 20,
            cfg_scale: 7.5,
            width: 512,
            height: 512,
            seed: 42,
            model_type: Some("test-model".to_string()),
            sampler: None,
            scheduler: None,
            loras: Vec::new(),
            bundle_id: Some("test-bundle".to_string()),
            model_component_id: "test-transformer".to_string(),
            t5_component_id: "test-t5".to_string(),
            clip_component_id: "test-clip".to_string(),
            vae_component_id: "test-vae".to_string(),
        }
    }

    #[tokio::test]
    async fn test_job_creation() {
        let manager = QueueManager::new(2);
        let params = create_test_params();

        let job_id = manager.add_job(params.clone()).await;

        assert!(!job_id.is_empty());

        let job = manager.get_job(&job_id).await;
        assert!(job.is_some());

        let job = job.unwrap();
        assert_eq!(job.id, job_id);
        assert_eq!(job.status, JobStatus::Pending);
        assert_eq!(job.progress, 0.0);
        assert_eq!(job.params.prompt, "test prompt");
        assert!(job.started_at.is_none());
        assert!(job.completed_at.is_none());
    }

    #[tokio::test]
    async fn test_job_status_updates() {
        let manager = QueueManager::new(2);
        let params = create_test_params();
        let job_id = manager.add_job(params).await;

        // Update to Running
        let result = manager.update_job_status(&job_id, JobStatus::Running).await;
        assert!(result.is_ok());

        let job = manager.get_job(&job_id).await.unwrap();
        assert_eq!(job.status, JobStatus::Running);
        assert!(job.started_at.is_some());

        // Update to Completed
        let result = manager.update_job_status(&job_id, JobStatus::Completed).await;
        assert!(result.is_ok());

        let job = manager.get_job(&job_id).await.unwrap();
        assert_eq!(job.status, JobStatus::Completed);
        assert_eq!(job.progress, 1.0);
        assert!(job.completed_at.is_some());

        // Try updating non-existent job
        let result = manager.update_job_status("nonexistent", JobStatus::Running).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Job not found"));
    }

    #[tokio::test]
    async fn test_job_progress_updates() {
        let manager = QueueManager::new(2);
        let params = create_test_params();
        let job_id = manager.add_job(params).await;

        // Update progress
        let result = manager.update_job_progress(&job_id, 0.5).await;
        assert!(result.is_ok());

        let job = manager.get_job(&job_id).await.unwrap();
        assert_eq!(job.progress, 0.5);

        // Try updating non-existent job
        let result = manager.update_job_progress("nonexistent", 0.75).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Job not found"));
    }

    #[tokio::test]
    async fn test_complete_job() {
        let manager = QueueManager::new(2);
        let params = create_test_params();
        let job_id = manager.add_job(params).await;

        let result = manager.complete_job(&job_id, "/path/to/result.png".to_string()).await;
        assert!(result.is_ok());

        let job = manager.get_job(&job_id).await.unwrap();
        assert_eq!(job.status, JobStatus::Completed);
        assert_eq!(job.progress, 1.0);
        assert_eq!(job.result_path, Some("/path/to/result.png".to_string()));
        assert!(job.completed_at.is_some());

        // Try completing non-existent job
        let result = manager.complete_job("nonexistent", "/path/to/result.png".to_string()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fail_job() {
        let manager = QueueManager::new(2);
        let params = create_test_params();
        let job_id = manager.add_job(params).await;

        let result = manager.fail_job(&job_id, "Test error".to_string()).await;
        assert!(result.is_ok());

        let job = manager.get_job(&job_id).await.unwrap();
        assert_eq!(job.status, JobStatus::Failed);
        assert_eq!(job.error, Some("Test error".to_string()));
        assert!(job.completed_at.is_some());

        // Try failing non-existent job
        let result = manager.fail_job("nonexistent", "error".to_string()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cancel_job() {
        let manager = QueueManager::new(2);
        let params = create_test_params();
        let job_id = manager.add_job(params).await;

        // Can cancel pending job
        let result = manager.cancel_job(&job_id).await;
        assert!(result);

        let job = manager.get_job(&job_id).await.unwrap();
        assert_eq!(job.status, JobStatus::Cancelled);
        assert!(job.completed_at.is_some());

        // Cannot cancel already cancelled job
        let result = manager.cancel_job(&job_id).await;
        assert!(!result);

        // Cannot cancel running job
        let params2 = create_test_params();
        let job_id2 = manager.add_job(params2).await;
        manager.update_job_status(&job_id2, JobStatus::Running).await.unwrap();

        let result = manager.cancel_job(&job_id2).await;
        assert!(!result);
    }

    #[tokio::test]
    async fn test_concurrent_job_tracking() {
        let manager = QueueManager::new(2);

        // Initially can start jobs
        assert!(manager.can_start_job().await);

        // Increment running count
        manager.increment_running().await;
        assert!(manager.can_start_job().await);

        manager.increment_running().await;
        assert!(!manager.can_start_job().await);

        // Decrement running count
        manager.decrement_running().await;
        assert!(manager.can_start_job().await);

        manager.decrement_running().await;
        assert!(manager.can_start_job().await);

        // Decrement below zero should be safe
        manager.decrement_running().await;
        assert!(manager.can_start_job().await);
    }

    #[tokio::test]
    async fn test_get_jobs_ordering() {
        let manager = QueueManager::new(2);

        let job_id1 = manager.add_job(create_test_params()).await;
        let job_id2 = manager.add_job(create_test_params()).await;
        let job_id3 = manager.add_job(create_test_params()).await;

        let jobs = manager.get_jobs().await;
        assert_eq!(jobs.len(), 3);
        assert_eq!(jobs[0].id, job_id1);
        assert_eq!(jobs[1].id, job_id2);
        assert_eq!(jobs[2].id, job_id3);
    }

    #[tokio::test]
    async fn test_clear_completed() {
        let manager = QueueManager::new(2);

        let job_id1 = manager.add_job(create_test_params()).await;
        let job_id2 = manager.add_job(create_test_params()).await;
        let job_id3 = manager.add_job(create_test_params()).await;

        manager.complete_job(&job_id1, "/path/1.png".to_string()).await.unwrap();
        manager.fail_job(&job_id2, "error".to_string()).await.unwrap();
        // job_id3 remains pending

        manager.clear_completed().await;

        let jobs = manager.get_jobs().await;
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].id, job_id3);

        // Verify removed jobs are really gone
        assert!(manager.get_job(&job_id1).await.is_none());
        assert!(manager.get_job(&job_id2).await.is_none());
        assert!(manager.get_job(&job_id3).await.is_some());
    }

    #[tokio::test]
    async fn test_hashmap_performance() {
        let manager = QueueManager::new(2);

        // Add many jobs
        let mut job_ids = Vec::new();
        for _ in 0..100 {
            let job_id = manager.add_job(create_test_params()).await;
            job_ids.push(job_id);
        }

        // Direct lookup should be O(1) with HashMap
        for job_id in &job_ids {
            let job = manager.get_job(job_id).await;
            assert!(job.is_some());
        }

        // Verify all jobs exist
        let all_jobs = manager.get_jobs().await;
        assert_eq!(all_jobs.len(), 100);
    }
}
