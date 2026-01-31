//! Utility functions for image processing and system monitoring

use std::path::Path;

pub fn ensure_dir_exists(path: &Path) -> std::io::Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

/// Get current GPU memory stats using nvidia-smi.
///
/// Returns (used_bytes, total_bytes, percent_used).
pub fn get_gpu_memory_stats() -> Option<(u64, u64, f32)> {
    let output = std::process::Command::new("nvidia-smi")
        .args([
            "--query-gpu=memory.used,memory.total",
            "--format=csv,noheader,nounits",
        ])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let line = stdout.lines().next().unwrap_or("");
            let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();

            if parts.len() >= 2 {
                let mem_used = parts[0].parse::<u64>().ok()?;
                let mem_total = parts[1].parse::<u64>().ok()?;
                let mem_percent = (mem_used as f64 / mem_total as f64 * 100.0) as f32;

                return Some((mem_used * 1024 * 1024, mem_total * 1024 * 1024, mem_percent));
            }
        }
        _ => {}
    }

    None
}

/// Format bytes into human-readable string.
pub fn format_bytes(bytes: u64) -> String {
    const GB: u64 = 1024 * 1024 * 1024;
    const MB: u64 = 1024 * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_ensure_dir_exists() {
        let temp_dir = env::temp_dir().join("flux_test");
        ensure_dir_exists(&temp_dir).unwrap();
        assert!(temp_dir.exists());
        std::fs::remove_dir(&temp_dir).unwrap();
    }
}
