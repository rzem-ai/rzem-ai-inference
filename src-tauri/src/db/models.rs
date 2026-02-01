use super::{InferenceDb, ComponentRecord, BundleRecord, ComponentInfo, BundleInfo, component_type_to_model_type, infer_family};
use rusqlite::params;
use anyhow::Result;
use serde::{Deserialize, Serialize};

// ========== Response DTOs (match frontend TypeScript types) ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelFileInfoResponse {
    pub id: String,
    pub model_id: String,
    pub path: String,
    pub resolved_path: String,
    pub sha256: Option<String>,
    pub size_bytes: i64,
    pub is_symlink: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExampleResponse {
    pub id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub example_type: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelPrefsBaseResponse {
    pub model_id: String,
    pub preferred_steps: Option<i32>,
    pub preferred_cfg: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelPrefsLoraResponse {
    pub model_id: String,
    pub strength_min: Option<f64>,
    pub strength_max: Option<f64>,
    pub strength_default: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfoResponse {
    pub id: String,
    pub model_type: String,
    pub family: String,
    pub display_name: String,
    pub description: Option<String>,
    pub files: Vec<ModelFileInfoResponse>,
    pub tags: Vec<String>,
    pub examples: Vec<ExampleResponse>,
    pub prefs_base: Option<ModelPrefsBaseResponse>,
    pub prefs_lora: Option<ModelPrefsLoraResponse>,
    pub trigger_words: Vec<String>,
    pub architecture: Option<String>,
    pub quantization: Option<String>,
    pub vram_mb: Option<i32>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleItemInfoResponse {
    pub id: String,
    pub model_id: String,
    pub role: String,
    pub model_display_name: String,
    pub model_family: String,
    pub model_type: String,
    pub model_vram_mb: Option<i32>,
    pub model_quantization: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleInfoResponse {
    pub id: String,
    pub display_name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub is_complete: bool,
    pub total_vram_mb: i32,
    pub tags: Vec<String>,
    pub items: Vec<BundleItemInfoResponse>,
    pub examples: Vec<ExampleResponse>,
    pub created_at: String,
    pub updated_at: String,
}

impl InferenceDb {
    /// Insert a model component
    pub fn insert_component(&self, comp: &ComponentRecord) -> Result<()> {
        let metadata_json = comp.metadata.as_ref()
            .map(|m| serde_json::to_string(m).ok())
            .flatten();

        self.conn.execute(
            "INSERT INTO model_components (
                id, component_type, format, file_path, file_size, file_hash, name,
                repo_id, repo_snapshot, architecture, quantization, supports_loras,
                is_sharded, shard_count, vram_mb, discovered_at, last_verified_at,
                is_available, metadata
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)
             ON CONFLICT(file_path) DO UPDATE SET
                name = excluded.name,
                component_type = excluded.component_type,
                format = excluded.format,
                file_size = excluded.file_size,
                last_verified_at = excluded.last_verified_at,
                is_available = excluded.is_available",
            params![
                comp.id,
                comp.component_type,
                comp.format,
                comp.file_path,
                comp.file_size,
                comp.file_hash,
                comp.name,
                comp.repo_id,
                comp.repo_snapshot,
                comp.architecture,
                comp.quantization,
                comp.supports_loras as i32,
                comp.is_sharded as i32,
                comp.shard_count,
                comp.vram_mb,
                comp.discovered_at,
                comp.last_verified_at,
                comp.is_available as i32,
                metadata_json,
            ],
        )?;

        Ok(())
    }

    /// Get component by ID
    pub fn get_component(&self, id: &str) -> Result<ComponentRecord> {
        self.conn.query_row(
            "SELECT id, component_type, format, file_path, file_size, file_hash, name,
                    repo_id, repo_snapshot, architecture, quantization, supports_loras,
                    is_sharded, shard_count, vram_mb, discovered_at, last_verified_at,
                    is_available, metadata
             FROM model_components WHERE id = ?1",
            params![id],
            |row| {
                let metadata_json: Option<String> = row.get(18)?;
                let metadata = metadata_json.and_then(|s| serde_json::from_str(&s).ok());

                Ok(ComponentRecord {
                    id: row.get(0)?,
                    component_type: row.get(1)?,
                    format: row.get(2)?,
                    file_path: row.get(3)?,
                    file_size: row.get(4)?,
                    file_hash: row.get(5)?,
                    name: row.get(6)?,
                    repo_id: row.get(7)?,
                    repo_snapshot: row.get(8)?,
                    architecture: row.get(9)?,
                    quantization: row.get(10)?,
                    supports_loras: row.get::<_, i32>(11)? != 0,
                    is_sharded: row.get::<_, i32>(12)? != 0,
                    shard_count: row.get(13)?,
                    vram_mb: row.get(14)?,
                    discovered_at: row.get(15)?,
                    last_verified_at: row.get(16)?,
                    is_available: row.get::<_, i32>(17)? != 0,
                    metadata,
                })
            },
        ).map_err(Into::into)
    }

    /// Get all components
    pub fn get_all_components(&self) -> Result<Vec<ComponentRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, component_type, format, file_path, file_size, file_hash, name,
                    repo_id, repo_snapshot, architecture, quantization, supports_loras,
                    is_sharded, shard_count, vram_mb, discovered_at, last_verified_at,
                    is_available, metadata
             FROM model_components ORDER BY name"
        )?;

        let components = stmt.query_map([], |row| {
            let metadata_json: Option<String> = row.get(18)?;
            let metadata = metadata_json.and_then(|s| serde_json::from_str(&s).ok());

            Ok(ComponentRecord {
                id: row.get(0)?,
                component_type: row.get(1)?,
                format: row.get(2)?,
                file_path: row.get(3)?,
                file_size: row.get(4)?,
                file_hash: row.get(5)?,
                name: row.get(6)?,
                repo_id: row.get(7)?,
                repo_snapshot: row.get(8)?,
                architecture: row.get(9)?,
                quantization: row.get(10)?,
                supports_loras: row.get::<_, i32>(11)? != 0,
                is_sharded: row.get::<_, i32>(12)? != 0,
                shard_count: row.get(13)?,
                vram_mb: row.get(14)?,
                discovered_at: row.get(15)?,
                last_verified_at: row.get(16)?,
                is_available: row.get::<_, i32>(17)? != 0,
                metadata,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(components)
    }

    /// Get components by type
    pub fn get_components_by_type(&self, comp_type: &str) -> Result<Vec<ComponentRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, component_type, format, file_path, file_size, file_hash, name,
                    repo_id, repo_snapshot, architecture, quantization, supports_loras,
                    is_sharded, shard_count, vram_mb, discovered_at, last_verified_at,
                    is_available, metadata
             FROM model_components WHERE component_type = ?1 ORDER BY name"
        )?;

        let components = stmt.query_map(params![comp_type], |row| {
            let metadata_json: Option<String> = row.get(18)?;
            let metadata = metadata_json.and_then(|s| serde_json::from_str(&s).ok());

            Ok(ComponentRecord {
                id: row.get(0)?,
                component_type: row.get(1)?,
                format: row.get(2)?,
                file_path: row.get(3)?,
                file_size: row.get(4)?,
                file_hash: row.get(5)?,
                name: row.get(6)?,
                repo_id: row.get(7)?,
                repo_snapshot: row.get(8)?,
                architecture: row.get(9)?,
                quantization: row.get(10)?,
                supports_loras: row.get::<_, i32>(11)? != 0,
                is_sharded: row.get::<_, i32>(12)? != 0,
                shard_count: row.get(13)?,
                vram_mb: row.get(14)?,
                discovered_at: row.get(15)?,
                last_verified_at: row.get(16)?,
                is_available: row.get::<_, i32>(17)? != 0,
                metadata,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(components)
    }

    /// Find a component by repo_id and component_type
    /// Used to find tokenizers associated with encoders from the same repo
    pub fn find_component_by_repo_and_type(&self, repo_id: &str, comp_type: &str) -> Result<ComponentRecord> {
        self.conn.query_row(
            "SELECT id, component_type, format, file_path, file_size, file_hash, name,
                    repo_id, repo_snapshot, architecture, quantization, supports_loras,
                    is_sharded, shard_count, vram_mb, discovered_at, last_verified_at,
                    is_available, metadata
             FROM model_components
             WHERE repo_id = ?1 AND component_type = ?2 AND is_available = 1
             LIMIT 1",
            params![repo_id, comp_type],
            |row| {
                let metadata_json: Option<String> = row.get(18)?;
                let metadata = metadata_json.and_then(|s| serde_json::from_str(&s).ok());

                Ok(ComponentRecord {
                    id: row.get(0)?,
                    component_type: row.get(1)?,
                    format: row.get(2)?,
                    file_path: row.get(3)?,
                    file_size: row.get(4)?,
                    file_hash: row.get(5)?,
                    name: row.get(6)?,
                    repo_id: row.get(7)?,
                    repo_snapshot: row.get(8)?,
                    architecture: row.get(9)?,
                    quantization: row.get(10)?,
                    supports_loras: row.get::<_, i32>(11)? != 0,
                    is_sharded: row.get::<_, i32>(12)? != 0,
                    shard_count: row.get(13)?,
                    vram_mb: row.get(14)?,
                    discovered_at: row.get(15)?,
                    last_verified_at: row.get(16)?,
                    is_available: row.get::<_, i32>(17)? != 0,
                    metadata,
                })
            },
        ).map_err(|e| anyhow::anyhow!("Component not found: {}", e))
    }

    /// Update component availability
    pub fn update_component_availability(&self, id: &str, available: bool) -> Result<()> {
        self.conn.execute(
            "UPDATE model_components SET is_available = ?1, last_verified_at = ?2 WHERE id = ?3",
            params![available as i32, chrono::Utc::now().timestamp(), id],
        )?;
        Ok(())
    }

    /// Check if a repo has been scanned with a specific snapshot hash
    /// Returns true if components from this repo+snapshot exist in the database
    pub fn has_repo_snapshot(&self, repo_id: &str, snapshot_hash: &str) -> Result<bool> {
        let count: i32 = self.conn.query_row(
            "SELECT COUNT(*) FROM model_components WHERE repo_id = ?1 AND repo_snapshot = ?2",
            params![repo_id, snapshot_hash],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// Insert a model bundle
    pub fn insert_bundle(&self, bundle: &BundleRecord) -> Result<()> {
        self.conn.execute(
            "INSERT INTO model_bundles (
                id, name, description, bundle_type, model_family, default_steps,
                default_guidance, step_min, step_max, total_vram_mb, is_complete,
                is_active, created_at, updated_at, validation_errors
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                bundle.id,
                bundle.name,
                bundle.description,
                bundle.bundle_type,
                bundle.model_family,
                bundle.default_steps,
                bundle.default_guidance,
                bundle.step_min,
                bundle.step_max,
                bundle.total_vram_mb,
                bundle.is_complete as i32,
                bundle.is_active as i32,
                bundle.created_at,
                bundle.updated_at,
                bundle.validation_errors,
            ],
        )?;

        Ok(())
    }

    /// Get bundle by ID with all component details
    pub fn get_bundle(&self, id: &str) -> Result<BundleInfo> {
        // Get bundle record
        let bundle: BundleRecord = self.conn.query_row(
            "SELECT id, name, description, bundle_type, model_family, default_steps,
                    default_guidance, step_min, step_max, total_vram_mb, is_complete,
                    is_active, created_at, updated_at, validation_errors
             FROM model_bundles WHERE id = ?1",
            params![id],
            |row| {
                Ok(BundleRecord {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    bundle_type: row.get(3)?,
                    model_family: row.get(4)?,
                    default_steps: row.get(5)?,
                    default_guidance: row.get(6)?,
                    step_min: row.get(7)?,
                    step_max: row.get(8)?,
                    total_vram_mb: row.get(9)?,
                    is_complete: row.get::<_, i32>(10)? != 0,
                    is_active: row.get::<_, i32>(11)? != 0,
                    created_at: row.get(12)?,
                    updated_at: row.get(13)?,
                    validation_errors: row.get(14)?,
                })
            },
        )?;

        // Get associated components
        let components = self.get_bundle_components(&bundle.id)?;

        Ok(BundleInfo {
            id: bundle.id,
            name: bundle.name,
            description: bundle.description,
            bundle_type: bundle.bundle_type,
            model_family: bundle.model_family,
            default_steps: bundle.default_steps,
            default_guidance: bundle.default_guidance,
            step_min: bundle.step_min,
            step_max: bundle.step_max,
            total_vram_mb: bundle.total_vram_mb,
            is_complete: bundle.is_complete,
            is_active: bundle.is_active,
            created_at: bundle.created_at,
            updated_at: bundle.updated_at,
            validation_errors: bundle.validation_errors,
            components,
        })
    }

    /// Get all bundles with component details
    pub fn get_all_bundles(&self) -> Result<Vec<BundleInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, bundle_type, model_family, default_steps,
                    default_guidance, step_min, step_max, total_vram_mb, is_complete,
                    is_active, created_at, updated_at, validation_errors
             FROM model_bundles ORDER BY name"
        )?;

        let bundle_records: Vec<BundleRecord> = stmt.query_map([], |row| {
            Ok(BundleRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                bundle_type: row.get(3)?,
                model_family: row.get(4)?,
                default_steps: row.get(5)?,
                default_guidance: row.get(6)?,
                step_min: row.get(7)?,
                step_max: row.get(8)?,
                total_vram_mb: row.get(9)?,
                is_complete: row.get::<_, i32>(10)? != 0,
                is_active: row.get::<_, i32>(11)? != 0,
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
                validation_errors: row.get(14)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        // Load components for each bundle
        let mut bundles = Vec::new();
        for bundle in bundle_records {
            let components = self.get_bundle_components(&bundle.id)?;
            bundles.push(BundleInfo {
                id: bundle.id,
                name: bundle.name,
                description: bundle.description,
                bundle_type: bundle.bundle_type,
                model_family: bundle.model_family,
                default_steps: bundle.default_steps,
                default_guidance: bundle.default_guidance,
                step_min: bundle.step_min,
                step_max: bundle.step_max,
                total_vram_mb: bundle.total_vram_mb,
                is_complete: bundle.is_complete,
                is_active: bundle.is_active,
                created_at: bundle.created_at,
                updated_at: bundle.updated_at,
                validation_errors: bundle.validation_errors,
                components,
            });
        }

        Ok(bundles)
    }

    /// Update bundle
    pub fn update_bundle(&self, id: &str, name: Option<&str>, description: Option<&str>) -> Result<()> {
        let now = chrono::Utc::now().timestamp();

        if let Some(n) = name {
            self.conn.execute(
                "UPDATE model_bundles SET name = ?1, updated_at = ?2 WHERE id = ?3",
                params![n, now, id],
            )?;
        }

        if let Some(d) = description {
            self.conn.execute(
                "UPDATE model_bundles SET description = ?1, updated_at = ?2 WHERE id = ?3",
                params![d, now, id],
            )?;
        }

        Ok(())
    }

    /// Delete bundle
    pub fn delete_bundle(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM model_bundles WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Set bundle as active (deactivates all others)
    pub fn set_bundle_active(&self, id: &str, active: bool) -> Result<()> {
        if active {
            // Deactivate all bundles first
            self.conn.execute("UPDATE model_bundles SET is_active = 0", [])?;
        }

        // Activate the specified bundle
        self.conn.execute(
            "UPDATE model_bundles SET is_active = ?1, updated_at = ?2 WHERE id = ?3",
            params![active as i32, chrono::Utc::now().timestamp(), id],
        )?;

        Ok(())
    }

    /// Deactivate all bundles
    pub fn deactivate_all_bundles(&self) -> Result<()> {
        self.conn.execute("UPDATE model_bundles SET is_active = 0", [])?;
        Ok(())
    }

    /// Get active bundle
    pub fn get_active_bundle(&self) -> Result<Option<BundleInfo>> {
        let result = self.conn.query_row(
            "SELECT id FROM model_bundles WHERE is_active = 1 LIMIT 1",
            [],
            |row| row.get::<_, String>(0),
        );

        match result {
            Ok(id) => Ok(Some(self.get_bundle(&id)?)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Add component to bundle
    pub fn add_component_to_bundle(&self, bundle_id: &str, comp_id: &str, role: &str, is_required: bool, priority: i32) -> Result<()> {
        self.conn.execute(
            "INSERT INTO bundle_components (bundle_id, component_id, component_role, is_required, priority)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(bundle_id, component_role, component_id) DO UPDATE SET
                is_required = excluded.is_required,
                priority = excluded.priority",
            params![bundle_id, comp_id, role, is_required as i32, priority],
        )?;
        Ok(())
    }

    /// Remove component from bundle by role
    pub fn remove_component_from_bundle(&self, bundle_id: &str, role: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM bundle_components WHERE bundle_id = ?1 AND component_role = ?2",
            params![bundle_id, role],
        )?;
        Ok(())
    }

    /// Get components for a bundle (with details)
    pub fn get_bundle_components(&self, bundle_id: &str) -> Result<Vec<ComponentInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT
                c.id, bc.component_role, c.name, c.component_type, c.format,
                c.file_path, c.file_size, c.quantization, c.vram_mb, c.is_available,
                bc.is_required, bc.priority
             FROM bundle_components bc
             JOIN model_components c ON bc.component_id = c.id
             WHERE bc.bundle_id = ?1
             ORDER BY bc.priority DESC, c.component_type"
        )?;

        let components = stmt.query_map(params![bundle_id], |row| {
            Ok(ComponentInfo {
                id: row.get(0)?,
                role: row.get(1)?,
                name: row.get(2)?,
                component_type: row.get(3)?,
                format: row.get(4)?,
                file_path: row.get(5)?,
                file_size: row.get(6)?,
                quantization: row.get(7)?,
                vram_mb: row.get(8)?,
                is_available: row.get::<_, i32>(9)? != 0,
                is_required: row.get::<_, i32>(10)? != 0,
                priority: row.get(11)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(components)
    }

    /// Convert a ComponentRecord into a ModelInfoResponse (for frontend API)
    fn component_to_model_info(&self, comp: &ComponentRecord) -> Result<ModelInfoResponse> {
        let tags = self.get_model_tags(&comp.id)?;
        let examples = self.get_examples("model", &comp.id)?;

        Ok(ModelInfoResponse {
            id: comp.id.clone(),
            model_type: component_type_to_model_type(&comp.component_type).to_string(),
            family: infer_family(comp.architecture.as_deref()).to_string(),
            display_name: comp.name.clone(),
            description: None,
            files: vec![ModelFileInfoResponse {
                id: comp.id.clone(),
                model_id: comp.id.clone(),
                path: comp.file_path.clone(),
                resolved_path: comp.file_path.clone(),
                sha256: comp.file_hash.clone(),
                size_bytes: comp.file_size,
                is_symlink: false,
            }],
            tags,
            examples,
            prefs_base: None,
            prefs_lora: None,
            trigger_words: vec![],
            architecture: comp.architecture.clone(),
            quantization: comp.quantization.clone(),
            vram_mb: comp.vram_mb,
            created_at: chrono::DateTime::from_timestamp(comp.discovered_at, 0)
                .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
                .unwrap_or_default(),
            updated_at: comp.last_verified_at
                .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0))
                .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
                .unwrap_or_default(),
        })
    }

    /// Get all models as frontend-compatible ModelInfoResponse
    pub fn get_all_models_response(&self) -> Result<Vec<ModelInfoResponse>> {
        let components = self.get_all_components()?;
        let mut models = Vec::with_capacity(components.len());
        for comp in &components {
            models.push(self.component_to_model_info(comp)?);
        }
        Ok(models)
    }

    /// Convert BundleInfo to BundleInfoResponse
    fn bundle_to_response(&self, bundle: &BundleInfo) -> Result<BundleInfoResponse> {
        let items: Vec<BundleItemInfoResponse> = bundle.components.iter().map(|c| {
            BundleItemInfoResponse {
                id: c.id.clone(),
                model_id: c.id.clone(),
                role: c.role.clone(),
                model_display_name: c.name.clone(),
                model_family: {
                    // Look up the component to get architecture for family inference
                    self.get_component(&c.id).ok()
                        .map(|comp| infer_family(comp.architecture.as_deref()).to_string())
                        .unwrap_or_else(|| "other".to_string())
                },
                model_type: component_type_to_model_type(&c.component_type).to_string(),
                model_vram_mb: c.vram_mb,
                model_quantization: c.quantization.clone(),
            }
        }).collect();

        let examples = self.get_examples("bundle", &bundle.id)?;

        Ok(BundleInfoResponse {
            id: bundle.id.clone(),
            display_name: bundle.name.clone(),
            description: bundle.description.clone(),
            is_active: bundle.is_active,
            is_complete: bundle.is_complete,
            total_vram_mb: bundle.total_vram_mb.unwrap_or(0),
            tags: vec![],
            items,
            examples,
            created_at: chrono::DateTime::from_timestamp(bundle.created_at, 0)
                .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
                .unwrap_or_default(),
            updated_at: chrono::DateTime::from_timestamp(bundle.updated_at, 0)
                .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
                .unwrap_or_default(),
        })
    }

    /// Get all bundles as frontend-compatible BundleInfoResponse
    pub fn get_all_bundles_response(&self) -> Result<Vec<BundleInfoResponse>> {
        let bundles = self.get_all_bundles()?;
        let mut responses = Vec::with_capacity(bundles.len());
        for bundle in &bundles {
            responses.push(self.bundle_to_response(bundle)?);
        }
        Ok(responses)
    }

    /// Update model display name
    pub fn update_model_name(&self, model_id: &str, name: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE model_components SET name = ?1 WHERE id = ?2",
            params![name, model_id],
        )?;
        Ok(())
    }

    /// Get tags for a model
    pub fn get_model_tags(&self, model_id: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT tag FROM model_tags WHERE model_id = ?1 ORDER BY tag"
        )?;
        let tags = stmt.query_map(params![model_id], |row| {
            row.get::<_, String>(0)
        })?.collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(tags)
    }

    /// Add a tag to a model
    pub fn add_model_tag(&self, model_id: &str, tag: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO model_tags (model_id, tag) VALUES (?1, ?2)",
            params![model_id, tag],
        )?;
        Ok(())
    }

    /// Remove a tag from a model
    pub fn remove_model_tag(&self, model_id: &str, tag: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM model_tags WHERE model_id = ?1 AND tag = ?2",
            params![model_id, tag],
        )?;
        Ok(())
    }

    /// Get examples for an entity
    pub fn get_examples(&self, entity_type: &str, entity_id: &str) -> Result<Vec<ExampleResponse>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, entity_type, entity_id, example_type, content, created_at
             FROM examples WHERE entity_type = ?1 AND entity_id = ?2 ORDER BY created_at"
        )?;
        let examples = stmt.query_map(params![entity_type, entity_id], |row| {
            Ok(ExampleResponse {
                id: row.get(0)?,
                entity_type: row.get(1)?,
                entity_id: row.get(2)?,
                example_type: row.get(3)?,
                content: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?.collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(examples)
    }

    /// Add an example
    pub fn add_example(&self, entity_type: &str, entity_id: &str, example_type: &str, content: &str) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        self.conn.execute(
            "INSERT INTO examples (id, entity_type, entity_id, example_type, content, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, entity_type, entity_id, example_type, content, now],
        )?;
        Ok(id)
    }

    /// Remove an example by ID
    pub fn remove_example(&self, example_id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM examples WHERE id = ?1",
            params![example_id],
        )?;
        Ok(())
    }

    /// Get compatible models for a given base model and target type
    pub fn get_compatible_models(&self, _base_model_id: &str, target_type: &str) -> Result<Vec<ModelInfoResponse>> {
        // Compatibility is family-based: return all models of target_type matching the base model's family
        let base = self.get_component(_base_model_id)?;
        let family = infer_family(base.architecture.as_deref());

        let all_components = self.get_all_components()?;
        let mut results = Vec::new();
        for comp in &all_components {
            let comp_model_type = component_type_to_model_type(&comp.component_type);
            let comp_family = infer_family(comp.architecture.as_deref());
            if comp_model_type == target_type && comp_family == family {
                results.push(self.component_to_model_info(comp)?);
            }
        }
        Ok(results)
    }

}
