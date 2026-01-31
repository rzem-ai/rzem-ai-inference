//! Bundle builder - auto-creates model bundles from discovered components

use anyhow::Result;
use std::collections::HashMap;
use tracing::{debug, info};

use super::scanner::{ComponentType, DiscoveredComponent};
use crate::gallery::{BundleRecord, ComponentRecord};

/// Bundle definition before database insertion
#[derive(Debug, Clone)]
pub struct BundleDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub bundle_type: String,
    pub model_family: String,
    pub default_steps: Option<i32>,
    pub default_guidance: Option<f64>,
    pub step_min: Option<i32>,
    pub step_max: Option<i32>,
    pub component_map: Vec<(String, String)>, // (role, component_id)
}

impl BundleDefinition {
    /// Convert to BundleRecord for database insertion
    pub fn to_record(&self, total_vram_mb: i32, is_complete: bool) -> BundleRecord {
        BundleRecord {
            id: self.id.clone(),
            name: self.name.clone(),
            description: Some(self.description.clone()),
            bundle_type: self.bundle_type.clone(),
            model_family: self.model_family.clone(),
            default_steps: self.default_steps,
            default_guidance: self.default_guidance,
            step_min: self.step_min,
            step_max: self.step_max,
            total_vram_mb: Some(total_vram_mb),
            is_complete,
            is_active: false,
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
            validation_errors: None,
        }
    }
}

/// Bundle builder for auto-discovering bundles
pub struct BundleBuilder {
    components: Vec<DiscoveredComponent>,
}

impl BundleBuilder {
    pub fn new(components: Vec<DiscoveredComponent>) -> Self {
        Self { components }
    }

    /// Auto-create bundles by grouping components from the same repo
    pub fn auto_create_bundles(&self) -> Vec<BundleDefinition> {
        let mut bundles = Vec::new();

        // Group components by repo_id
        let by_repo = self.group_by_repo();

        for (repo_id, components) in by_repo {
            // Try to create full-precision bundle
            if let Some(bundle) = self.create_full_bundle(&repo_id, &components) {
                debug!(bundle_id = %bundle.id, "Created full-precision bundle");
                bundles.push(bundle);
            }

            // Try to create quantized bundle
            if let Some(bundle) = self.create_quantized_bundle(&repo_id, &components) {
                debug!(bundle_id = %bundle.id, "Created quantized bundle");
                bundles.push(bundle);
            }

            // Try to create mixed bundle (quantized transformer + full precision text encoders)
            if let Some(bundle) = self.create_mixed_bundle(&repo_id, &components) {
                debug!(bundle_id = %bundle.id, "Created mixed bundle");
                bundles.push(bundle);
            }
        }

        info!(count = bundles.len(), "Auto-created bundles");
        bundles
    }

    /// Group components by repository ID
    fn group_by_repo(&self) -> HashMap<String, Vec<&DiscoveredComponent>> {
        let mut by_repo: HashMap<String, Vec<&DiscoveredComponent>> = HashMap::new();

        for comp in &self.components {
            by_repo
                .entry(comp.repo_id.clone())
                .or_default()
                .push(comp);
        }

        by_repo
    }

    /// Create full-precision bundle (no quantization)
    fn create_full_bundle(&self, repo_id: &str, comps: &[&DiscoveredComponent]) -> Option<BundleDefinition> {
        // Find transformer (no quantization)
        let transformer = comps.iter().find(|c|
            c.component_type == ComponentType::Transformer
            && c.quantization.is_none()
        )?;

        // Find support components (prefer from same repo, fallback to any)
        let t5 = self.find_best_component(comps, ComponentType::T5Encoder, false)?;
        let clip = self.find_best_component(comps, ComponentType::ClipEncoder, false)?;
        let vae = self.find_best_component(comps, ComponentType::VaeDecoder, false)?;

        // Find tokenizers (optional - fallback to any available)
        let t5_tokenizer = self.find_best_component(comps, ComponentType::T5Tokenizer, false);
        let clip_tokenizer = self.find_best_component(comps, ComponentType::ClipTokenizer, false);

        let model_family = infer_family(&transformer.architecture);
        let (step_min, step_max, default_steps, default_guidance) = get_model_defaults(&model_family);

        let mut component_map = vec![
            ("transformer".to_string(), component_id(transformer)),
            ("t5".to_string(), component_id(t5)),
            ("clip".to_string(), component_id(clip)),
            ("vae".to_string(), component_id(vae)),
        ];

        // Add tokenizers if found
        if let Some(t5_tok) = t5_tokenizer {
            component_map.push(("t5_tokenizer".to_string(), component_id(t5_tok)));
        }
        if let Some(clip_tok) = clip_tokenizer {
            component_map.push(("clip_tokenizer".to_string(), component_id(clip_tok)));
        }

        Some(BundleDefinition {
            id: format!("{}-full", sanitize_id(repo_id)),
            name: format!("{} (Full Precision)", display_name(&transformer.architecture)),
            description: format!("Complete {} bundle with full-precision components from {}", model_family, repo_id),
            bundle_type: "auto_discovered".to_string(),
            model_family,
            default_steps: Some(default_steps),
            default_guidance: Some(default_guidance),
            step_min: Some(step_min),
            step_max: Some(step_max),
            component_map,
        })
    }

    /// Create fully quantized bundle
    fn create_quantized_bundle(&self, repo_id: &str, comps: &[&DiscoveredComponent]) -> Option<BundleDefinition> {
        // Find quantized transformer
        let transformer = comps.iter().find(|c|
            c.component_type == ComponentType::Transformer
            && c.quantization.is_some()
        )?;

        // Find quantized T5 or fallback to full
        let t5 = self.find_best_component(comps, ComponentType::T5Encoder, true)?;

        // CLIP and VAE typically don't have quantized versions in same repo
        let clip = self.find_best_component(comps, ComponentType::ClipEncoder, false)?;
        let vae = self.find_best_component(comps, ComponentType::VaeDecoder, false)?;

        // Find tokenizers (optional - fallback to any available)
        let t5_tokenizer = self.find_best_component(comps, ComponentType::T5Tokenizer, false);
        let clip_tokenizer = self.find_best_component(comps, ComponentType::ClipTokenizer, false);

        let model_family = infer_family(&transformer.architecture);
        let (step_min, step_max, default_steps, default_guidance) = get_model_defaults(&model_family);

        let quant_label = transformer.quantization.as_ref()
            .map(|q| format!(" ({})", q))
            .unwrap_or_default();

        let mut component_map = vec![
            ("transformer".to_string(), component_id(transformer)),
            ("t5".to_string(), component_id(t5)),
            ("clip".to_string(), component_id(clip)),
            ("vae".to_string(), component_id(vae)),
        ];

        // Add tokenizers if found
        if let Some(t5_tok) = t5_tokenizer {
            component_map.push(("t5_tokenizer".to_string(), component_id(t5_tok)));
        }
        if let Some(clip_tok) = clip_tokenizer {
            component_map.push(("clip_tokenizer".to_string(), component_id(clip_tok)));
        }

        Some(BundleDefinition {
            id: format!("{}-quantized", sanitize_id(repo_id)),
            name: format!("{}{}", display_name(&transformer.architecture), quant_label),
            description: format!("Memory-efficient {} bundle with quantized components from {}", model_family, repo_id),
            bundle_type: "auto_discovered".to_string(),
            model_family,
            default_steps: Some(default_steps),
            default_guidance: Some(default_guidance),
            step_min: Some(step_min),
            step_max: Some(step_max),
            component_map,
        })
    }

    /// Create mixed bundle (quantized transformer, full text encoders)
    fn create_mixed_bundle(&self, repo_id: &str, comps: &[&DiscoveredComponent]) -> Option<BundleDefinition> {
        // Only create if we have both quantized transformer AND full precision text encoders
        let transformer = comps.iter().find(|c|
            c.component_type == ComponentType::Transformer
            && c.quantization.is_some()
        )?;

        let t5_full = comps.iter().find(|c|
            c.component_type == ComponentType::T5Encoder
            && c.quantization.is_none()
        )?;

        // Don't create mixed bundle if we don't have full T5
        if t5_full.repo_id != repo_id {
            return None;
        }

        let clip = self.find_best_component(comps, ComponentType::ClipEncoder, false)?;
        let vae = self.find_best_component(comps, ComponentType::VaeDecoder, false)?;

        // Find tokenizers (optional - fallback to any available)
        let t5_tokenizer = self.find_best_component(comps, ComponentType::T5Tokenizer, false);
        let clip_tokenizer = self.find_best_component(comps, ComponentType::ClipTokenizer, false);

        let model_family = infer_family(&transformer.architecture);
        let (step_min, step_max, default_steps, default_guidance) = get_model_defaults(&model_family);

        let mut component_map = vec![
            ("transformer".to_string(), component_id(transformer)),
            ("t5".to_string(), component_id(t5_full)),
            ("clip".to_string(), component_id(clip)),
            ("vae".to_string(), component_id(vae)),
        ];

        // Add tokenizers if found
        if let Some(t5_tok) = t5_tokenizer {
            component_map.push(("t5_tokenizer".to_string(), component_id(t5_tok)));
        }
        if let Some(clip_tok) = clip_tokenizer {
            component_map.push(("clip_tokenizer".to_string(), component_id(clip_tok)));
        }

        Some(BundleDefinition {
            id: format!("{}-mixed", sanitize_id(repo_id)),
            name: format!("{} (Balanced)", display_name(&transformer.architecture)),
            description: format!("Balanced {} bundle with quantized transformer and full-precision text encoders from {}", model_family, repo_id),
            bundle_type: "auto_discovered".to_string(),
            model_family,
            default_steps: Some(default_steps),
            default_guidance: Some(default_guidance),
            step_min: Some(step_min),
            step_max: Some(step_max),
            component_map,
        })
    }

    /// Find best component of a type (prefer from same repo, optionally quantized)
    fn find_best_component<'a>(&'a self, repo_comps: &[&'a DiscoveredComponent], comp_type: ComponentType, prefer_quantized: bool) -> Option<&'a DiscoveredComponent> {
        // First try to find in the same repo with matching quantization preference
        if let Some(&comp) = repo_comps.iter().find(|&&c|
            c.component_type == comp_type
            && (c.quantization.is_some() == prefer_quantized)
        ) {
            return Some(comp);
        }

        // Fallback: any component of this type from same repo
        if let Some(&comp) = repo_comps.iter().find(|&&c| c.component_type == comp_type) {
            return Some(comp);
        }

        // Last resort: any component of this type from any repo
        self.components.iter().find(|c|
            c.component_type == comp_type
            && (c.quantization.is_some() == prefer_quantized)
        ).or_else(|| {
            self.components.iter().find(|c| c.component_type == comp_type)
        })
    }
}

/// Convert DiscoveredComponent to ComponentRecord for database
pub fn to_component_record(comp: &DiscoveredComponent) -> ComponentRecord {
    ComponentRecord {
        id: component_id(comp),
        component_type: comp.component_type.as_str().to_string(),
        format: match comp.format {
            super::scanner::ModelFormat::Safetensors => "safetensors".to_string(),
            super::scanner::ModelFormat::Gguf => "gguf".to_string(),
            super::scanner::ModelFormat::Json => "json".to_string(),
        },
        file_path: comp.path.to_string_lossy().to_string(),
        file_size: comp.file_size as i64,
        file_hash: None,
        name: generate_component_name(comp),
        repo_id: Some(comp.repo_id.clone()),
        repo_snapshot: comp.repo_snapshot.clone(),
        architecture: Some(comp.architecture.clone()),
        quantization: comp.quantization.clone(),
        supports_loras: comp.component_type == ComponentType::Transformer
            && comp.format == super::scanner::ModelFormat::Safetensors
            && comp.quantization.is_none(),
        is_sharded: comp.is_sharded,
        shard_count: comp.shard_count.map(|c| c as i32),
        vram_mb: comp.vram_mb.map(|v| v as i32),
        discovered_at: chrono::Utc::now().timestamp(),
        last_verified_at: Some(chrono::Utc::now().timestamp()),
        is_available: comp.path.exists(),
        metadata: Some(comp.metadata.clone()),
    }
}

/// Generate unique component ID
fn component_id(comp: &DiscoveredComponent) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    comp.path.hash(&mut hasher);
    format!("comp-{:x}", hasher.finish())
}

/// Generate display name for component using the filename
fn generate_component_name(comp: &DiscoveredComponent) -> String {
    // Use filename as the base name
    let filename = comp.path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    // Remove extension
    let name = filename
        .strip_suffix(".safetensors")
        .or_else(|| filename.strip_suffix(".gguf"))
        .or_else(|| filename.strip_suffix(".json"))
        .unwrap_or(filename);

    // For sharded models (directory path), use directory name
    if comp.is_sharded {
        let dir_name = comp.path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("sharded");
        return format!("{} [Sharded]", dir_name);
    }

    name.to_string()
}

/// Sanitize repo ID for use as bundle ID
fn sanitize_id(repo_id: &str) -> String {
    repo_id.replace('/', "-").replace('.', "-").to_lowercase()
}

/// Get display name from architecture
fn display_name(architecture: &str) -> String {
    if architecture.contains("schnell") {
        "FLUX.1 Schnell".to_string()
    } else if architecture.contains("dev") {
        "FLUX.1 Dev".to_string()
    } else if architecture.contains("z-image") {
        "Z-Image Turbo".to_string()
    } else {
        "FLUX Model".to_string()
    }
}

/// Infer model family from architecture
fn infer_family(architecture: &str) -> String {
    if architecture.contains("z-image") {
        "zindex".to_string()
    } else {
        "flux".to_string()
    }
}

/// Get default parameters for model family
fn get_model_defaults(family: &str) -> (i32, i32, i32, f64) {
    // (step_min, step_max, default_steps, default_guidance)
    match family {
        "flux" => (1, 50, 4, 3.5),
        "zindex" => (1, 50, 4, 3.5),
        _ => (1, 100, 20, 7.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_id() {
        assert_eq!(
            sanitize_id("black-forest-labs/FLUX.1-schnell"),
            "black-forest-labs-flux-1-schnell"
        );
    }

    #[test]
    fn test_infer_family() {
        assert_eq!(infer_family("flux-schnell"), "flux");
        assert_eq!(infer_family("z-image-turbo"), "zindex");
    }

    #[test]
    fn test_model_defaults() {
        let (min, max, def, guide) = get_model_defaults("flux");
        assert_eq!(min, 1);
        assert_eq!(max, 50);
        assert_eq!(def, 4);
        assert_eq!(guide, 3.5);
    }
}
