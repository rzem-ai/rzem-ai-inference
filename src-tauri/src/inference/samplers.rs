//! Sampler and Scheduler implementations for FLUX diffusion
//!
//! This module provides different sampling algorithms and noise schedules
//! for the FLUX image generation pipeline.
//!
//! ## Samplers
//! - **Euler**: First-order ODE solver, fast and stable
//! - **EulerA**: Euler Ancestral - adds noise at each step for more variation
//! - **DPM++ 2M**: Second-order multistep method, higher quality at same step count
//!
//! ## Schedulers
//! - **Normal**: Linear timestep schedule (default FLUX behavior)
//! - **Simple**: Basic uniform schedule, common baseline
//! - **Karras**: Noise schedule from Karras et al., smoother denoising
//! - **Exponential**: Exponential decay schedule

use serde::{Deserialize, Serialize};

/// Sampling algorithm for the denoising process
#[derive(Clone, Copy, Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SamplerType {
    /// Standard Euler method - first-order ODE solver
    /// Fast, stable, good default choice
    #[default]
    Euler,
    /// Euler Ancestral - adds noise at each step
    /// Produces more variation, can be more creative
    EulerA,
    /// DPM++ 2M - second-order multistep method
    /// Higher quality results at the same step count
    DpmPP2M,
}

impl SamplerType {
    /// Get display name for UI
    pub fn display_name(&self) -> &'static str {
        match self {
            SamplerType::Euler => "Euler",
            SamplerType::EulerA => "Euler A",
            SamplerType::DpmPP2M => "DPM++ 2M",
        }
    }

    /// Get all available samplers
    pub fn all() -> &'static [SamplerType] {
        &[SamplerType::Euler, SamplerType::EulerA, SamplerType::DpmPP2M]
    }
}

impl std::str::FromStr for SamplerType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "euler" => Ok(SamplerType::Euler),
            "euler_a" | "eulera" | "euler-a" => Ok(SamplerType::EulerA),
            "dpm_pp_2m" | "dpmpp2m" | "dpm++2m" | "dpm_pp2m" => Ok(SamplerType::DpmPP2M),
            _ => Err(format!("Unknown sampler: {}", s)),
        }
    }
}

impl std::fmt::Display for SamplerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SamplerType::Euler => write!(f, "euler"),
            SamplerType::EulerA => write!(f, "euler_a"),
            SamplerType::DpmPP2M => write!(f, "dpm_pp_2m"),
        }
    }
}

/// Noise schedule type for timestep generation
#[derive(Clone, Copy, Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SchedulerType {
    /// Linear timestep schedule - default FLUX behavior
    #[default]
    Normal,
    /// Simple uniform schedule - basic baseline, evenly spaced
    Simple,
    /// Karras noise schedule - smoother denoising curve
    /// From "Elucidating the Design Space of Diffusion-Based Generative Models"
    Karras,
    /// Exponential decay schedule
    Exponential,
}

impl SchedulerType {
    /// Get display name for UI
    pub fn display_name(&self) -> &'static str {
        match self {
            SchedulerType::Normal => "Normal",
            SchedulerType::Simple => "Simple",
            SchedulerType::Karras => "Karras",
            SchedulerType::Exponential => "Exponential",
        }
    }

    /// Get all available schedulers
    pub fn all() -> &'static [SchedulerType] {
        &[SchedulerType::Normal, SchedulerType::Simple, SchedulerType::Karras, SchedulerType::Exponential]
    }
}

impl std::str::FromStr for SchedulerType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "normal" | "linear" => Ok(SchedulerType::Normal),
            "simple" | "uniform" => Ok(SchedulerType::Simple),
            "karras" => Ok(SchedulerType::Karras),
            "exponential" | "exp" => Ok(SchedulerType::Exponential),
            _ => Err(format!("Unknown scheduler: {}", s)),
        }
    }
}

impl std::fmt::Display for SchedulerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchedulerType::Normal => write!(f, "normal"),
            SchedulerType::Simple => write!(f, "simple"),
            SchedulerType::Karras => write!(f, "karras"),
            SchedulerType::Exponential => write!(f, "exponential"),
        }
    }
}

/// Generate timestep schedule based on scheduler type
///
/// Returns a vector of sigma values from 1.0 (full noise) to 0.0 (no noise)
/// with length `steps + 1` (includes both endpoints)
pub fn get_timesteps(steps: usize, scheduler: SchedulerType) -> Vec<f64> {
    match scheduler {
        SchedulerType::Normal => linear_schedule(steps),
        SchedulerType::Simple => simple_schedule(steps),
        SchedulerType::Karras => karras_schedule(steps),
        SchedulerType::Exponential => exponential_schedule(steps),
    }
}

/// Linear schedule - evenly spaced timesteps from 1.0 to 0.0
/// This matches FLUX's default behavior
fn linear_schedule(steps: usize) -> Vec<f64> {
    (0..=steps)
        .map(|i| 1.0 - (i as f64 / steps as f64))
        .collect()
}

/// Simple uniform schedule - basic evenly-spaced timesteps
/// Similar to linear but commonly used as a baseline in diffusion UIs
fn simple_schedule(steps: usize) -> Vec<f64> {
    // Simple uniform distribution from 1.0 to 0.0
    // Uses n+1 points to include both endpoints
    (0..=steps)
        .map(|i| {
            let t = i as f64 / steps as f64;
            1.0 - t
        })
        .collect()
}

/// Karras noise schedule from the paper:
/// "Elucidating the Design Space of Diffusion-Based Generative Models"
///
/// σ_i = (σ_max^(1/ρ) + i/(n-1) * (σ_min^(1/ρ) - σ_max^(1/ρ)))^ρ
///
/// Uses ρ=7 as recommended in the paper for image generation
fn karras_schedule(steps: usize) -> Vec<f64> {
    let sigma_min: f64 = 0.0;
    let sigma_max: f64 = 1.0;
    let rho: f64 = 7.0;

    let inv_rho = 1.0 / rho;
    let sigma_max_inv_rho = sigma_max.powf(inv_rho);
    let sigma_min_inv_rho = sigma_min.powf(inv_rho);

    (0..=steps)
        .map(|i| {
            let t = i as f64 / steps as f64;
            let sigma_inv_rho = sigma_max_inv_rho + t * (sigma_min_inv_rho - sigma_max_inv_rho);
            sigma_inv_rho.powf(rho)
        })
        .collect()
}

/// Exponential decay schedule
///
/// σ_i = σ_max * (σ_min/σ_max)^(i/(n-1))
fn exponential_schedule(steps: usize) -> Vec<f64> {
    let sigma_min: f64 = 0.001; // Small epsilon to avoid log(0)
    let sigma_max: f64 = 1.0;
    let ratio = sigma_min / sigma_max;

    let mut schedule: Vec<f64> = (0..steps)
        .map(|i| {
            let t = i as f64 / (steps - 1).max(1) as f64;
            sigma_max * ratio.powf(t)
        })
        .collect();

    // Ensure we end at exactly 0
    schedule.push(0.0);
    schedule
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sampler_type_display() {
        assert_eq!(SamplerType::Euler.to_string(), "euler");
        assert_eq!(SamplerType::EulerA.to_string(), "euler_a");
        assert_eq!(SamplerType::DpmPP2M.to_string(), "dpm_pp_2m");
    }

    #[test]
    fn test_sampler_type_parse() {
        assert_eq!("euler".parse::<SamplerType>().unwrap(), SamplerType::Euler);
        assert_eq!("euler_a".parse::<SamplerType>().unwrap(), SamplerType::EulerA);
        assert_eq!("dpm_pp_2m".parse::<SamplerType>().unwrap(), SamplerType::DpmPP2M);
        assert_eq!("dpmpp2m".parse::<SamplerType>().unwrap(), SamplerType::DpmPP2M);
    }

    #[test]
    fn test_scheduler_type_display() {
        assert_eq!(SchedulerType::Normal.to_string(), "normal");
        assert_eq!(SchedulerType::Karras.to_string(), "karras");
        assert_eq!(SchedulerType::Exponential.to_string(), "exponential");
    }

    #[test]
    fn test_scheduler_type_parse() {
        assert_eq!("normal".parse::<SchedulerType>().unwrap(), SchedulerType::Normal);
        assert_eq!("karras".parse::<SchedulerType>().unwrap(), SchedulerType::Karras);
        assert_eq!("exponential".parse::<SchedulerType>().unwrap(), SchedulerType::Exponential);
    }

    #[test]
    fn test_linear_schedule() {
        let schedule = linear_schedule(4);
        assert_eq!(schedule.len(), 5);
        assert!((schedule[0] - 1.0).abs() < 1e-10);
        assert!((schedule[4] - 0.0).abs() < 1e-10);
        // Check it's monotonically decreasing
        for i in 1..schedule.len() {
            assert!(schedule[i] <= schedule[i-1]);
        }
    }

    #[test]
    fn test_karras_schedule() {
        let schedule = karras_schedule(4);
        assert_eq!(schedule.len(), 5);
        assert!((schedule[0] - 1.0).abs() < 1e-10);
        assert!((schedule[4] - 0.0).abs() < 1e-10);
        // Check it's monotonically decreasing
        for i in 1..schedule.len() {
            assert!(schedule[i] <= schedule[i-1]);
        }
    }

    #[test]
    fn test_exponential_schedule() {
        let schedule = exponential_schedule(4);
        assert_eq!(schedule.len(), 5);
        assert!((schedule[0] - 1.0).abs() < 1e-10);
        assert!((schedule[4] - 0.0).abs() < 1e-10);
        // Check it's monotonically decreasing
        for i in 1..schedule.len() {
            assert!(schedule[i] <= schedule[i-1]);
        }
    }

    #[test]
    fn test_get_timesteps_uses_correct_scheduler() {
        let linear = get_timesteps(4, SchedulerType::Normal);
        let karras = get_timesteps(4, SchedulerType::Karras);

        // They should have the same endpoints
        assert!((linear[0] - karras[0]).abs() < 1e-10);
        assert!((linear[4] - karras[4]).abs() < 1e-10);

        // But different middle values (Karras spends more time in low-noise regime)
        assert!((linear[2] - karras[2]).abs() > 0.01);
    }

    #[test]
    fn test_serde_sampler() {
        let sampler = SamplerType::EulerA;
        let json = serde_json::to_string(&sampler).unwrap();
        assert_eq!(json, "\"euler_a\"");
        let parsed: SamplerType = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, SamplerType::EulerA);
    }

    #[test]
    fn test_serde_scheduler() {
        let scheduler = SchedulerType::Karras;
        let json = serde_json::to_string(&scheduler).unwrap();
        assert_eq!(json, "\"karras\"");
        let parsed: SchedulerType = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, SchedulerType::Karras);
    }
}
