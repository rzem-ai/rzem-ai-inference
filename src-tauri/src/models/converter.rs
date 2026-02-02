//! Model format converter - converts ComfyUI/InvokeAI format to native FLUX format

use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use tracing::{info, warn};
use safetensors::SafeTensors;
use std::fs;

/// Convert a ComfyUI/InvokeAI format model to native FLUX format
///
/// This strips the `model.diffusion_model.` prefix from all tensor names,
/// making the model compatible with Candle-based inference.
pub fn convert_comfyui_to_native(
    input_path: &Path,
    output_path: Option<&Path>,
) -> Result<PathBuf> {
    info!("🔄 Starting ComfyUI → Native FLUX conversion");
    info!("   Input: {}", input_path.display());

    // Validate input file exists
    if !input_path.exists() {
        anyhow::bail!("Input file does not exist: {}", input_path.display());
    }

    // Determine output path
    let output = if let Some(out) = output_path {
        out.to_path_buf()
    } else {
        // Default: add "-native" suffix before extension
        let stem = input_path.file_stem()
            .and_then(|s| s.to_str())
            .context("Invalid input filename")?;
        let parent = input_path.parent()
            .context("Cannot determine parent directory")?;
        parent.join(format!("{}-native.safetensors", stem))
    };

    info!("   Output: {}", output.display());

    // Read input file
    info!("   📖 Reading input file...");
    let buffer = fs::read(input_path)?;

    let file_size_mb = buffer.len() / 1_000_000;
    info!("   File size: {} MB", file_size_mb);

    // Parse safetensors
    info!("   🔍 Parsing safetensors format...");
    let safetensors = SafeTensors::deserialize(&buffer)
        .context("Failed to parse safetensors file")?;

    // Check if it's ComfyUI format
    let tensor_names = safetensors.names();
    let total_tensors = tensor_names.len();
    info!("   Found {} tensors", total_tensors);

    let comfyui_count = tensor_names.iter()
        .filter(|n| n.as_str().starts_with("model.diffusion_model."))
        .count();

    if comfyui_count == 0 {
        warn!("   ⚠️  No ComfyUI format tensors found (model.diffusion_model.* prefix)");
        warn!("   This file may already be in native format or is an unsupported format");
        anyhow::bail!("Input file does not appear to be in ComfyUI format");
    }

    info!("   ✓ Detected ComfyUI format ({}/{} tensors have prefix)", comfyui_count, total_tensors);

    // Convert by creating a modified buffer with renamed tensors
    info!("   🔄 Converting tensor names and writing output...");

    const PREFIX: &str = "model.diffusion_model.";

    // Create a new safetensors file with renamed keys
    // We'll read the original file, parse the header, rename keys, and rebuild
    convert_and_write(&buffer, &output, PREFIX)?;

    let output_size = fs::metadata(&output)?.len();
    let output_size_mb = output_size / 1_000_000;
    info!("   ✓ Output written: {} MB", output_size_mb);

    info!("✅ Conversion complete!");
    info!("   Native format saved to: {}", output.display());

    Ok(output)
}

/// Convert safetensors file by renaming tensor keys
fn convert_and_write(input_buffer: &[u8], output_path: &Path, prefix_to_strip: &str) -> Result<()> {
    use std::io::Write;

    // Parse the input
    let st = SafeTensors::deserialize(input_buffer)?;

    // Read header to get tensor info
    // Header is: [8 bytes: header_len as u64 LE][header_len bytes: JSON][tensor data]
    let header_len = u64::from_le_bytes(input_buffer[0..8].try_into()?) as usize;
    let header_bytes = &input_buffer[8..8+header_len];
    let header: serde_json::Value = serde_json::from_slice(header_bytes)?;

    let header_map = header.as_object()
        .context("Header is not a JSON object")?;

    // Build new header with renamed keys
    let mut new_header = serde_json::Map::new();

    // Copy metadata if exists
    if let Some(metadata) = header_map.get("__metadata__") {
        new_header.insert("__metadata__".to_string(), metadata.clone());
    }

    // Rename tensor entries
    let data_start = 8 + header_len;
    for (old_name, tensor_info) in header_map {
        if old_name == "__metadata__" {
            continue;
        }

        let new_name = if old_name.starts_with(prefix_to_strip) {
            old_name.strip_prefix(prefix_to_strip).unwrap().to_string()
        } else {
            old_name.clone()
        };

        new_header.insert(new_name, tensor_info.clone());
    }

    // Serialize new header
    let new_header_json = serde_json::to_string(&new_header)?;
    let new_header_bytes = new_header_json.as_bytes();
    let new_header_len = new_header_bytes.len() as u64;

    // Write output file
    let mut output_file = fs::File::create(output_path)?;

    // Write new header length
    output_file.write_all(&new_header_len.to_le_bytes())?;

    // Write new header
    output_file.write_all(new_header_bytes)?;

    // Copy tensor data as-is (data doesn't change, only names)
    output_file.write_all(&input_buffer[data_start..])?;

    Ok(())
}

/// Check if a file is in ComfyUI format
pub fn is_comfyui_format(path: &Path) -> Result<bool> {
    let buffer = fs::read(path)?;
    let safetensors = SafeTensors::deserialize(&buffer)?;
    let has_prefix = safetensors.names()
        .iter()
        .any(|n| n.as_str().starts_with("model.diffusion_model."));

    Ok(has_prefix)
}
