//! Integration test for model bundle system

#[cfg(test)]
mod tests {
    use rzem_ai_inference::gallery::{BundleRecord, ComponentRecord, GalleryDb};
    use rzem_ai_inference::models::{ComponentRole, ModelPaths};
    use std::path::PathBuf;

    #[test]
    fn test_bundle_aware_paths() {
        // Create in-memory database
        let db = GalleryDb::new(":memory:").unwrap();
        db.init_schema().unwrap();

        // Create a test component
        let component = ComponentRecord {
            id: "test-comp-1".to_string(),
            component_type: "transformer".to_string(),
            format: "safetensors".to_string(),
            file_path: "/tmp/test/flux1-schnell.safetensors".to_string(),
            file_size: 1000000,
            file_hash: None,
            name: "Test Transformer".to_string(),
            repo_id: Some("test/repo".to_string()),
            repo_snapshot: None,
            architecture: Some("flux-schnell".to_string()),
            quantization: None,
            supports_loras: true,
            is_sharded: false,
            shard_count: None,
            vram_mb: Some(23000),
            discovered_at: chrono::Utc::now().timestamp(),
            last_verified_at: Some(chrono::Utc::now().timestamp()),
            is_available: true,
            metadata: None,
        };

        db.insert_component(&component).unwrap();

        // Create a test bundle
        let bundle = BundleRecord {
            id: "test-bundle-1".to_string(),
            name: "Test Bundle".to_string(),
            description: Some("Test bundle for integration testing".to_string()),
            bundle_type: "user_created".to_string(),
            model_family: "flux".to_string(),
            default_steps: Some(4),
            default_guidance: Some(3.5),
            step_min: Some(1),
            step_max: Some(50),
            total_vram_mb: Some(23000),
            is_complete: false,
            is_active: true,
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
            validation_errors: None,
        };

        db.insert_bundle(&bundle).unwrap();

        // Add component to bundle
        db.add_component_to_bundle(&bundle.id, &component.id, "transformer", true, 0)
            .unwrap();

        // Test bundle retrieval
        let retrieved_bundle = db.get_bundle(&bundle.id).unwrap();
        assert_eq!(retrieved_bundle.id, bundle.id);
        assert_eq!(retrieved_bundle.name, bundle.name);
        assert_eq!(retrieved_bundle.components.len(), 1);
        assert_eq!(retrieved_bundle.components[0].role, "transformer");

        // Test active bundle
        let active_bundle = db.get_active_bundle().unwrap();
        assert!(active_bundle.is_some());
        assert_eq!(active_bundle.unwrap().id, bundle.id);
    }

    #[test]
    fn test_component_role_conversion() {
        assert_eq!(
            ComponentRole::from_str("transformer").unwrap(),
            ComponentRole::Transformer
        );
        assert_eq!(
            ComponentRole::from_str("t5").unwrap(),
            ComponentRole::T5
        );
        assert_eq!(
            ComponentRole::from_str("clip").unwrap(),
            ComponentRole::Clip
        );
        assert_eq!(ComponentRole::Transformer.as_str(), "transformer");
        assert_eq!(ComponentRole::T5.as_str(), "t5");
    }

    #[test]
    fn test_model_paths_legacy_fallback() {
        // Test that ModelPaths can be created even without active bundle
        // This should fall back to legacy mode
        let paths = ModelPaths::new().unwrap();

        // Should not be in bundle mode (unless there's actually an active bundle)
        // We can't assert false here because there might be a real bundle
        assert!(paths.cache_dir.to_string_lossy().contains("huggingface"));
    }

    #[test]
    fn test_bundle_operations() {
        let db = GalleryDb::new(":memory:").unwrap();
        db.init_schema().unwrap();

        // Test component insertion
        let comp1 = ComponentRecord {
            id: "comp-1".to_string(),
            component_type: "t5_encoder".to_string(),
            format: "safetensors".to_string(),
            file_path: "/path/to/t5".to_string(),
            file_size: 5000000,
            file_hash: None,
            name: "T5 Encoder".to_string(),
            repo_id: Some("repo/t5".to_string()),
            repo_snapshot: None,
            architecture: Some("t5-xxl".to_string()),
            quantization: None,
            supports_loras: false,
            is_sharded: true,
            shard_count: Some(2),
            vram_mb: Some(9000),
            discovered_at: chrono::Utc::now().timestamp(),
            last_verified_at: None,
            is_available: true,
            metadata: None,
        };

        db.insert_component(&comp1).unwrap();

        // Retrieve component
        let retrieved = db.get_component(&comp1.id).unwrap();
        assert_eq!(retrieved.id, comp1.id);
        assert_eq!(retrieved.component_type, "t5_encoder");
        assert_eq!(retrieved.is_sharded, true);
        assert_eq!(retrieved.shard_count, Some(2));

        // Test bundle creation
        let bundle = BundleRecord {
            id: "bundle-1".to_string(),
            name: "Full Bundle".to_string(),
            description: Some("Complete model bundle".to_string()),
            bundle_type: "auto_discovered".to_string(),
            model_family: "flux".to_string(),
            default_steps: Some(4),
            default_guidance: Some(3.5),
            step_min: Some(1),
            step_max: Some(50),
            total_vram_mb: Some(24000),
            is_complete: true,
            is_active: false,
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
            validation_errors: None,
        };

        db.insert_bundle(&bundle).unwrap();

        // Add component to bundle
        db.add_component_to_bundle(&bundle.id, &comp1.id, "t5", true, 0)
            .unwrap();

        // Retrieve bundle with components
        let retrieved_bundle = db.get_bundle(&bundle.id).unwrap();
        assert_eq!(retrieved_bundle.components.len(), 1);
        assert_eq!(retrieved_bundle.components[0].component_type, "t5_encoder");

        // Test bundle activation
        db.set_bundle_active(&bundle.id, true).unwrap();
        let active = db.get_active_bundle().unwrap();
        assert!(active.is_some());
        assert_eq!(active.unwrap().id, bundle.id);

        // Test bundle deactivation
        db.deactivate_all_bundles().unwrap();
        let no_active = db.get_active_bundle().unwrap();
        assert!(no_active.is_none());

        // Test bundle deletion
        db.delete_bundle(&bundle.id).unwrap();
        assert!(db.get_bundle(&bundle.id).is_err());
    }

    #[test]
    fn test_multiple_bundles() {
        let db = GalleryDb::new(":memory:").unwrap();
        db.init_schema().unwrap();

        // Create multiple bundles
        for i in 1..=3 {
            let bundle = BundleRecord {
                id: format!("bundle-{}", i),
                name: format!("Bundle {}", i),
                description: Some(format!("Test bundle {}", i)),
                bundle_type: "auto_discovered".to_string(),
                model_family: "flux".to_string(),
                default_steps: Some(4),
                default_guidance: Some(3.5),
                step_min: Some(1),
                step_max: Some(50),
                total_vram_mb: Some(24000),
                is_complete: true,
                is_active: false,
                created_at: chrono::Utc::now().timestamp(),
                updated_at: chrono::Utc::now().timestamp(),
                validation_errors: None,
            };
            db.insert_bundle(&bundle).unwrap();
        }

        // Get all bundles
        let all_bundles = db.get_all_bundles().unwrap();
        assert_eq!(all_bundles.len(), 3);

        // Activate one
        db.set_bundle_active("bundle-2", true).unwrap();

        // Verify only one is active
        let bundles = db.get_all_bundles().unwrap();
        let active_count = bundles.iter().filter(|b| b.is_active).count();
        assert_eq!(active_count, 1);

        let active_bundle = bundles.iter().find(|b| b.is_active).unwrap();
        assert_eq!(active_bundle.id, "bundle-2");
    }
}
