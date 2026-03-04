//! `xybrid pack` command handler.

use anyhow::{Context, Result};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use xybrid_core::bundler::XyBundle;

/// Package model artifacts into a .xyb bundle.
///
/// Scans `./models/<name>/` (or a custom path) and writes
/// `./dist/<name>-<version>-<target>.xyb`.
pub(crate) fn pack_model(
    name: &str,
    version: &str,
    target: &str,
    custom_path: Option<&Path>,
) -> Result<()> {
    println!("📦 Xybrid Packager");
    println!("{}", "=".repeat(60));
    println!("Model: {}", name);
    println!("Version: {}", version);
    println!("Target: {}", target);
    println!();

    let models_dir = resolve_source_dir(name, custom_path)?;

    if !models_dir.exists() || !models_dir.is_dir() {
        return Err(anyhow::anyhow!(
            "Model directory not found: {}",
            models_dir.display()
        ));
    }

    println!("📂 Source: {}", models_dir.display());

    let mut dist_dir = std::env::current_dir().context("Failed to get current directory")?;
    dist_dir.push("dist");
    if !dist_dir.exists() {
        fs::create_dir_all(&dist_dir).context("Failed to create dist directory")?;
    }

    let out_path = dist_dir.join(format!("{}-{}-{}.xyb", name, version, target));

    let mut bundle = XyBundle::new(name, version, target);

    let metadata_path = models_dir.join("model_metadata.json");
    if metadata_path.exists() {
        println!("🔍 Found model_metadata.json - including in bundle");
    }

    let mut files_to_add = Vec::new();
    visit_dir(&models_dir, &mut files_to_add)?;

    let (added, duplicates) = add_files_to_bundle(&mut bundle, &files_to_add)?;

    if !duplicates.is_empty() {
        println!("⚠️  Skipped duplicate filenames (consider flattening tree):");
        for d in &duplicates {
            println!("   - {}", d);
        }
        println!();
    }

    if added == 0 {
        return Err(anyhow::anyhow!(
            "No files found to add in {}",
            models_dir.display()
        ));
    }

    bundle
        .write(&out_path)
        .with_context(|| format!("Failed to write bundle: {}", out_path.display()))?;

    print_bundle_summary(&bundle, &out_path);

    Ok(())
}

fn resolve_source_dir(name: &str, custom_path: Option<&Path>) -> Result<std::path::PathBuf> {
    if let Some(custom_path) = custom_path {
        Ok(custom_path.to_path_buf())
    } else {
        let mut dir = std::env::current_dir().context("Failed to get current directory")?;
        dir.push("models");
        dir.push(name);
        Ok(dir)
    }
}

fn visit_dir(dir: &Path, files: &mut Vec<std::path::PathBuf>) -> Result<()> {
    for entry in
        fs::read_dir(dir).with_context(|| format!("Failed to read dir: {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            visit_dir(&path, files)?;
        } else if path.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

fn add_files_to_bundle(
    bundle: &mut XyBundle,
    files: &[std::path::PathBuf],
) -> Result<(usize, Vec<String>)> {
    let mut seen = HashSet::new();
    let mut duplicates = Vec::new();
    let mut added = 0usize;

    for file in files {
        let fname = file
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        if fname.is_empty() {
            continue;
        }
        if seen.contains(&fname) {
            duplicates.push(fname);
            continue;
        }
        seen.insert(fname);
        bundle
            .add_file(file)
            .with_context(|| format!("Failed to add file: {}", file.display()))?;
        added += 1;
    }

    duplicates.sort();
    duplicates.dedup();
    Ok((added, duplicates))
}

fn print_bundle_summary(bundle: &XyBundle, out_path: &Path) {
    println!("✅ Bundle created: {}", out_path.display());
    println!("   Model ID: {}", bundle.manifest().model_id);
    println!("   Version:  {}", bundle.manifest().version);
    println!("   Target:   {}", bundle.manifest().target);
    println!("   Files:    {}", bundle.manifest().files.len());
    println!("   Hash:     {}", bundle.manifest().hash);

    if bundle.manifest().has_metadata {
        println!("   Metadata: ✅ Included (metadata-driven execution enabled)");
    } else {
        println!("   Metadata: ⚠️  Not found (consider adding model_metadata.json)");
    }
    println!();
}
