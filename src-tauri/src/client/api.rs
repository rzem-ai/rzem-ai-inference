//! REST API client for remote server communication

use super::ClientConfig;
use crate::queue::{GenerationParams, GenerationJob};
use crate::chatbot::{RefineRequest, ChatResponse};
use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// REST API client
pub struct ApiClient {
    client: Client,
    config: ClientConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateResponse {
    pub job_id: String,
    pub status: String,
    pub queue_position: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

impl ApiClient {
    /// Create a new API client
    pub fn new(config: ClientConfig) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self { client, config }
    }

    /// Test connection to server
    pub async fn health_check(&self) -> Result<HealthResponse> {
        let url = format!("{}/health", self.config.api_base());
        debug!(url = %url, "Health check request");

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to connect to server: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!("Server returned error: {}", response.status()));
        }

        let health = response.json::<HealthResponse>().await
            .map_err(|e| anyhow!("Failed to parse health response: {}", e))?;

        info!(status = %health.status, version = %health.version, "Server health OK");
        Ok(health)
    }

    /// Submit a generation job
    pub async fn submit_job(&self, params: GenerationParams) -> Result<String> {
        let url = format!("{}/generate", self.config.api_base());
        debug!(url = %url, prompt = %params.prompt, "Submitting job");

        let response = self.client
            .post(&url)
            .json(&params)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to submit job: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("Server returned error {}: {}", status, body));
        }

        let result = response.json::<GenerateResponse>().await
            .map_err(|e| anyhow!("Failed to parse response: {}", e))?;

        info!(job_id = %result.job_id, "Job submitted successfully");
        Ok(result.job_id)
    }

    /// Get all jobs in the queue
    pub async fn get_jobs(&self) -> Result<Vec<GenerationJob>> {
        let url = format!("{}/queue", self.config.api_base());
        debug!(url = %url, "Getting queue jobs");

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to get jobs: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!("Server returned error: {}", response.status()));
        }

        let jobs = response.json::<Vec<GenerationJob>>().await
            .map_err(|e| anyhow!("Failed to parse jobs: {}", e))?;

        debug!(count = jobs.len(), "Retrieved jobs");
        Ok(jobs)
    }

    /// Get a specific job by ID
    pub async fn get_job(&self, job_id: &str) -> Result<Option<GenerationJob>> {
        let url = format!("{}/queue/{}", self.config.api_base(), job_id);
        debug!(url = %url, job_id = %job_id, "Getting job");

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to get job: {}", e))?;

        if response.status().as_u16() == 404 {
            return Ok(None);
        }

        if !response.status().is_success() {
            return Err(anyhow!("Server returned error: {}", response.status()));
        }

        let job = response.json::<GenerationJob>().await
            .map_err(|e| anyhow!("Failed to parse job: {}", e))?;

        Ok(Some(job))
    }

    /// Cancel a job
    pub async fn cancel_job(&self, job_id: &str) -> Result<bool> {
        let url = format!("{}/queue/{}", self.config.api_base(), job_id);
        debug!(url = %url, job_id = %job_id, "Cancelling job");

        let response = self.client
            .delete(&url)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to cancel job: {}", e))?;

        if response.status().as_u16() == 400 {
            // Job cannot be cancelled (already running/completed)
            return Ok(false);
        }

        if !response.status().is_success() {
            return Err(anyhow!("Server returned error: {}", response.status()));
        }

        info!(job_id = %job_id, "Job cancelled successfully");
        Ok(true)
    }

    /// Download a file from the server
    pub async fn download_file(&self, filename: &str) -> Result<Vec<u8>> {
        let url = format!("{}/files/{}", self.config.api_base(), filename);
        debug!(url = %url, filename = %filename, "Downloading file");

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to download file: {}", e))?;

        if response.status().as_u16() == 404 {
            return Err(anyhow!("File not found: {}", filename));
        }

        if !response.status().is_success() {
            return Err(anyhow!("Server returned error: {}", response.status()));
        }

        let bytes = response.bytes().await
            .map_err(|e| anyhow!("Failed to read file data: {}", e))?;

        info!(filename = %filename, size = bytes.len(), "File downloaded");
        Ok(bytes.to_vec())
    }

    /// Get the full URL for a file
    pub fn get_file_url(&self, filename: &str) -> String {
        format!("{}/files/{}", self.config.api_base(), filename)
    }

    /// Refine a prompt using the chatbot
    pub async fn chat_refine(&self, request: RefineRequest) -> Result<ChatResponse> {
        let url = format!("{}/chat/refine", self.config.api_base());
        debug!(url = %url, "Sending chat refine request");

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send chat request: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("Server returned error {}: {}", status, body));
        }

        let result = response.json::<ChatResponse>().await
            .map_err(|e| anyhow!("Failed to parse response: {}", e))?;

        Ok(result)
    }
}

/// Test connection to server
pub async fn test_connection(config: &ClientConfig) -> Result<()> {
    let client = ApiClient::new(config.clone());
    client.health_check().await?;
    Ok(())
}
