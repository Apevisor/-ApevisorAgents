//! `xybrid cache` command handler.

use anyhow::{Context, Result};
use colored::*;
use std::fs;

use super::types::CacheCommand;
use super::utils::{dir_size_bytes, format_size};

/// Handle `xybrid cache` subcommands.
pub(crate) fn handle_cache_command(command: CacheCommand) -> Result<()> {
    let mut client = xybrid_sdk::registry_client::RegistryClient::from_env()
        .context("Failed to initialize registry client")?;

    match command {
        CacheCommand::List => list_cache(&client),
        CacheCommand::Status => show_cache_status(&client),
        CacheCommand::Clear { model_id } => clear_cache(&mut client, model_id),
    }
}

fn list_cache(client: &xybrid_sdk::registry_client::RegistryClient) -> Result<()> {
    println!("📦 Xybrid Model Cache");
    println!("{}", "=".repeat(60));
    println!();

    let stats = client.cache_stats().context("Failed to get cache stats")?;

    println!("📂 Cache directory: {}", stats.cache_path.display());
    println!();

    if stats.model_count == 0 {
        println!("ℹ️  Cache is empty.");
        println!("   Use 'xybrid fetch --model <id>' to download models.");
        return Ok(());
    }

    if stats.cache_path.exists() {
        for entry in fs::read_dir(&stats.cache_path)? {
            let entry = entry?;
            if entry.path().is_dir() {
                let model_name = entry.file_name();
                let model_name = model_name.to_string_lossy();
                let model_size = dir_size_bytes(&entry.path()).unwrap_or(0);
                let size_str = format_size(model_size);

                println!(
                    "  {} {} ({})",
                    "•".bright_cyan(),
                    model_name.cyan().bold(),
                    size_str.bright_black()
                );
            }
        }
    }

    println!();
    println!("{}", "=".repeat(60));
    println!(
        "Total: {} models, {}",
        stats.model_count,
        stats.total_size_human()
    );

    Ok(())
}

fn show_cache_status(client: &xybrid_sdk::registry_client::RegistryClient) -> Result<()> {
    println!("📊 Xybrid Cache Status");
    println!("{}", "=".repeat(60));
    println!();

    let stats = client.cache_stats().context("Failed to get cache stats")?;

    println!("  Cache Directory: {}", stats.cache_path.display());
    println!("  Cached Models:   {}", stats.model_count);
    println!(
        "  Total Size:      {}",
        stats.total_size_human().bright_cyan()
    );

    if !stats.cache_path.exists() {
        println!();
        println!("  ℹ️  Cache directory does not exist yet.");
        println!("     It will be created when you download your first model.");
    }

    println!();
    println!("{}", "=".repeat(60));

    Ok(())
}

fn clear_cache(
    client: &mut xybrid_sdk::registry_client::RegistryClient,
    model_id: Option<String>,
) -> Result<()> {
    if let Some(id) = model_id {
        println!("🗑️  Clearing cache for: {}", id.cyan().bold());
        println!("{}", "=".repeat(60));
        println!();

        client
            .clear_cache(&id)
            .context(format!("Failed to clear cache for '{}'", id))?;

        println!("✅ Cache cleared for model '{}'", id);
    } else {
        println!("🗑️  Clearing entire model cache");
        println!("{}", "=".repeat(60));
        println!();

        println!("⚠️  This will delete ALL cached models.");
        println!("   Press Enter to continue or Ctrl+C to cancel...");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok();

        client.clear_all_cache().context("Failed to clear cache")?;

        println!("✅ All cached models cleared");
    }

    println!();
    println!("{}", "=".repeat(60));

    Ok(())
}
