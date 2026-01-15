//! Utility functions for image processing and system monitoring

use std::path::Path;

pub fn ensure_dir_exists(path: &Path) -> std::io::Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
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
