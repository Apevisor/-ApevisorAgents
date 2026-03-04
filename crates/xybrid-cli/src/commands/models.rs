//! `xybrid models` command handlers.

use anyhow::{Context, Result};
use colored::*;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use xybrid_core::bundler::XyBundle;
use xybrid_core::execution_template::ModelMetadata;
use xybrid_sdk::registry_client::RegistryClient;

use super::types::ModelsCommand;
use super::utils::{format_params, format_size};

/// Handle `xybrid models` subcommands.
pub(crate) fn handle_models_command(command: ModelsCommand) -> Result<()> {
    let client = RegistryClient::from_env().context("Failed to initialize registry client")?;

    match command {
        ModelsCommand::List => list_models(&client),
        ModelsCommand::Search { query } => search_models(&client, &query),
        ModelsCommand::Info { model_id } => show_model_info(&client, &model_id),
        ModelsCommand::Voices { model_id } => handle_voices_command(&client, &model_id),
    }
}

fn list_models(client: &RegistryClient) -> Result<()> {
    println!("📦 Xybrid Model Registry");
    println!("{}", "=".repeat(60));
    println!();

    let models = client
        .list_models()
        .context("Failed to list models from registry")?;

    if models.is_empty() {
        println!("ℹ️  No models found in registry.");
        return Ok(());
    }

    let mut by_task: BTreeMap<String, Vec<_>> = BTreeMap::new();
    for model in &models {
        by_task.entry(model.task.clone()).or_default().push(model);
    }

    for (task, task_models) in by_task {
        println!("{}", format!("📁 {}", task.to_uppercase()).cyan().bold());
        println!();

        for model in task_models {
            let params_str = format_params(model.parameters);
            println!("  {} {}", "•".bright_cyan(), model.id.cyan().bold());
            println!(
                "    {} {} | {} params",
                model.family.bright_black(),
                "|".bright_black(),
                params_str.bright_black()
            );
            println!("    {}", model.description.bright_black());
            if !model.variants.is_empty() {
                println!("    Variants: {}", model.variants.join(", ").bright_green());
            }
            println!();
        }
    }

    println!("{}", "=".repeat(60));
    println!("Total: {} models", models.len());

    Ok(())
}

fn search_models(client: &RegistryClient, query: &str) -> Result<()> {
    println!("🔍 Searching for: {}", query.cyan().bold());
    println!("{}", "=".repeat(60));
    println!();

    let models = client
        .list_models()
        .context("Failed to list models from registry")?;

    let query_lower = query.to_lowercase();
    let matches: Vec<_> = models
        .iter()
        .filter(|m| {
            m.id.to_lowercase().contains(&query_lower)
                || m.family.to_lowercase().contains(&query_lower)
                || m.task.to_lowercase().contains(&query_lower)
                || m.description.to_lowercase().contains(&query_lower)
        })
        .collect();

    if matches.is_empty() {
        println!("ℹ️  No models found matching '{}'", query);
        return Ok(());
    }

    for model in matches.iter() {
        let params_str = format_params(model.parameters);
        println!("  {} {}", "•".bright_cyan(), model.id.cyan().bold());
        println!(
            "    {} | {} | {} params",
            model.task.bright_magenta(),
            model.family.bright_black(),
            params_str.bright_black()
        );
        println!("    {}", model.description.bright_black());
        println!();
    }

    println!("{}", "=".repeat(60));
    println!("Found: {} models", matches.len());

    Ok(())
}

fn show_model_info(client: &RegistryClient, model_id: &str) -> Result<()> {
    println!("📋 Model Details: {}", model_id.cyan().bold());
    println!("{}", "=".repeat(60));
    println!();

    let model = client
        .get_model(model_id)
        .context(format!("Failed to get model '{}'", model_id))?;

    println!("  ID:          {}", model.id.cyan().bold());
    println!("  Family:      {}", model.family);
    println!("  Task:        {}", model.task.bright_magenta());
    println!("  Parameters:  {}", format_params(model.parameters));
    println!("  Description: {}", model.description);
    println!();

    if let Some(default) = &model.default_variant {
        println!("  Default Variant: {}", default.bright_green());
    }

    if !model.variants.is_empty() {
        println!();
        println!("  {} Variants:", "📦".bright_cyan());
        for (name, info) in &model.variants {
            let size_str = format_size(info.size_bytes);
            println!(
                "    {} {} ({}, {})",
                "•".bright_cyan(),
                name.bright_green(),
                info.platform,
                size_str.bright_black()
            );
            println!(
                "      Format: {} | Quantization: {}",
                info.format.bright_blue(),
                info.quantization.bright_yellow()
            );
        }
    }

    if model.task.to_lowercase().contains("tts")
        || model.task.to_lowercase().contains("text-to-speech")
    {
        println!();
        println!(
            "  💡 This is a TTS model. Use '{}' to see available voices.",
            format!("xybrid models voices {}", model_id).bright_cyan()
        );
    }

    println!();
    println!("{}", "=".repeat(60));

    Ok(())
}

/// Handle `xybrid models voices <model-id>` command.
fn handle_voices_command(client: &RegistryClient, model_id: &str) -> Result<()> {
    println!("🎤 Voices for: {}", model_id.cyan().bold());
    println!("{}", "=".repeat(60));
    println!();

    let model = client
        .get_model(model_id)
        .context(format!("Failed to get model '{}'", model_id))?;

    if !model.task.to_lowercase().contains("tts")
        && !model.task.to_lowercase().contains("text-to-speech")
    {
        println!(
            "ℹ️  Model '{}' is not a TTS model (task: {}).",
            model_id, model.task
        );
        println!("   Voice selection is only available for text-to-speech models.");
        return Ok(());
    }

    let resolved = client
        .resolve(model_id, None)
        .context(format!("Failed to resolve model '{}'", model_id))?;

    let bundle_path = if client.is_cached(model_id, None).unwrap_or(false) {
        client.get_cache_path(&resolved)
    } else {
        println!("📥 Downloading model to read voice catalog...");
        println!();

        use indicatif::{ProgressBar, ProgressStyle};

        let pb = ProgressBar::new(resolved.size_bytes);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes}")
                .unwrap()
                .progress_chars("█▓▒░  "),
        );

        let path = client.fetch(model_id, None, |progress| {
            let bytes_done = (progress * resolved.size_bytes as f32) as u64;
            pb.set_position(bytes_done);
        })?;

        pb.finish_and_clear();
        path
    };

    let mut metadata = load_metadata_from_bundle(&bundle_path)?;
    metadata = try_local_fixtures_fallback(metadata, model_id);

    if !metadata.has_voices() {
        print_no_voices_hint(model_id);
        return Ok(());
    }

    let voices = metadata.list_voices();
    println!(
        "Found {} voices for {}",
        voices.len().to_string().bright_green(),
        model_id.cyan()
    );
    println!();

    print_voices_by_language(&voices);

    if let Some(default) = metadata.default_voice() {
        println!(
            "Default voice: {} ({})",
            default.name.bright_green(),
            default.id
        );
    }

    println!();
    println!("{}", "=".repeat(60));
    println!();
    println!("Usage:");
    println!(
        "  {} --model {} --input-text \"Hello\" --voice {}",
        "xybrid run".bright_cyan(),
        model_id,
        "<voice-id>".bright_yellow()
    );
    println!();

    Ok(())
}

fn load_metadata_from_bundle(bundle_path: &Path) -> Result<ModelMetadata> {
    if bundle_path.is_dir() {
        let metadata_path = bundle_path.join("model_metadata.json");
        if !metadata_path.exists() {
            anyhow::bail!(
                "model_metadata.json not found at {}",
                metadata_path.display()
            );
        }
        let content = fs::read_to_string(&metadata_path)?;
        return Ok(serde_json::from_str(&content)?);
    }

    if bundle_path.extension().is_some_and(|ext| ext == "xyb") {
        let bundle = XyBundle::load(bundle_path)?;
        let metadata_json = bundle.get_metadata_json()?.ok_or_else(|| {
            anyhow::anyhow!(
                "model_metadata.json not found in bundle at {}",
                bundle_path.display()
            )
        })?;
        return Ok(serde_json::from_str(&metadata_json)?);
    }

    let metadata_path = bundle_path.join("model_metadata.json");
    if !metadata_path.exists() {
        anyhow::bail!(
            "model_metadata.json not found at {}",
            metadata_path.display()
        );
    }
    let content = fs::read_to_string(&metadata_path)?;
    Ok(serde_json::from_str(&content)?)
}

fn try_local_fixtures_fallback(mut metadata: ModelMetadata, model_id: &str) -> ModelMetadata {
    if metadata.has_voices() {
        return metadata;
    }

    let fixtures_path = PathBuf::from("integration-tests/fixtures/models")
        .join(model_id)
        .join("model_metadata.json");

    if fixtures_path.exists() {
        if let Ok(content) = fs::read_to_string(&fixtures_path) {
            if let Ok(local_metadata) = serde_json::from_str::<ModelMetadata>(&content) {
                if local_metadata.has_voices() {
                    println!("📂 Using voice catalog from local fixtures");
                    println!("   (Registry bundle may need updating)");
                    println!();
                    metadata = local_metadata;
                }
            }
        }
    }

    metadata
}

fn print_no_voices_hint(model_id: &str) {
    println!("ℹ️  Model '{}' does not have a voice catalog.", model_id);
    println!();
    println!("   This TTS model may use a single default voice, or the");
    println!("   registry bundle needs to be updated with voice info.");
    println!();
    println!("   For local development with Kokoro, run:");
    println!("     ./integration-tests/download.sh kokoro-82m");
    println!("     cargo run -p xybrid-core --example tts_kokoro -- --list-voices");
}

fn print_voices_by_language(voices: &[&xybrid_core::execution_template::VoiceInfo]) {
    let mut by_language: BTreeMap<String, Vec<_>> = BTreeMap::new();
    for voice in voices {
        let lang = voice.language.as_deref().unwrap_or("unknown").to_string();
        by_language.entry(lang).or_default().push(voice);
    }

    for (language, lang_voices) in by_language {
        let flag = match language.as_str() {
            "en-US" => "🇺🇸",
            "en-GB" => "🇬🇧",
            "ja-JP" => "🇯🇵",
            "zh-CN" => "🇨🇳",
            "de-DE" => "🇩🇪",
            "fr-FR" => "🇫🇷",
            "es-ES" => "🇪🇸",
            _ => "🌐",
        };

        println!(
            "{} {} ({} voices)",
            flag,
            language.bright_cyan().bold(),
            lang_voices.len()
        );
        println!("{}", "─".repeat(55));
        println!(
            "  {:<15} {:<12} {:<8} {}",
            "ID".bright_black(),
            "Name".bright_black(),
            "Gender".bright_black(),
            "Style".bright_black()
        );
        println!("{}", "─".repeat(55));

        for voice in lang_voices {
            let gender_icon = match voice.gender.as_deref() {
                Some("female") => "♀",
                Some("male") => "♂",
                _ => " ",
            };
            println!(
                "  {:<15} {:<12} {} {:<6} {}",
                voice.id.cyan(),
                voice.name,
                gender_icon,
                voice.gender.as_deref().unwrap_or("-"),
                voice.style.as_deref().unwrap_or("neutral").bright_black()
            );
        }
        println!();
    }
}
