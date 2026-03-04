//! `xybrid trace` command handler and telemetry log analysis.

use anyhow::{Context, Result};
use colored::*;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

/// Parsed telemetry log entry.
#[derive(Debug, Clone)]
struct TelemetryLogEntry {
    timestamp: u64,
    severity: String,
    event: String,
    #[allow(dead_code)]
    message: String,
    stage: Option<String>,
    target: Option<String>,
    latency_ms: Option<u32>,
    allowed: Option<bool>,
    reason: Option<String>,
}

/// Trace and analyze telemetry logs from a session.
pub(crate) fn trace_session(session: Option<String>, export_path: Option<&Path>) -> Result<()> {
    println!("📊 Xybrid Trace Analyzer");
    println!("{}", "=".repeat(60));
    println!();

    let traces_dir = get_traces_directory()?;

    if let Some(session_id) = &session {
        analyze_session(&traces_dir, session_id, export_path)?;
    } else {
        list_sessions(&traces_dir)?;
    }

    Ok(())
}

fn analyze_session(traces_dir: &Path, session_id: &str, export_path: Option<&Path>) -> Result<()> {
    println!("🔍 Loading telemetry for session: {}", session_id);
    println!();

    let trace_file = traces_dir.join(format!("{}.log", session_id));

    if !trace_file.exists() {
        return Err(anyhow::anyhow!(
            "Session '{}' not found.\n  Looked in: {}",
            session_id,
            trace_file.display()
        ));
    }

    let entries = read_telemetry_log(&trace_file)?;

    if entries.is_empty() {
        println!("⚠️  No telemetry entries found in session.");
        return Ok(());
    }

    display_telemetry_table(&entries);
    display_summary(&entries);

    if let Some(export_path) = export_path {
        export_trace_summary(&entries, export_path)?;
        println!("💾 Trace summary exported to: {}", export_path.display());
        println!();
    }

    Ok(())
}

/// Get the traces directory path (~/.xybrid/traces/).
pub(crate) fn get_traces_directory() -> Result<PathBuf> {
    let mut path =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
    path.push(".xybrid");
    path.push("traces");

    if !path.exists() {
        fs::create_dir_all(&path)
            .with_context(|| format!("Failed to create traces directory: {}", path.display()))?;
    }

    Ok(path)
}

/// Find the latest session ID from the traces directory.
pub(crate) fn find_latest_session() -> Result<Option<String>> {
    let traces_dir = get_traces_directory()?;

    if !traces_dir.exists() {
        return Ok(None);
    }

    let mut sessions = collect_sessions(&traces_dir)?;

    if sessions.is_empty() {
        return Ok(None);
    }

    sessions.sort_by(|a, b| b.1.cmp(&a.1));
    Ok(sessions.first().map(|(id, _)| id.clone()))
}

fn collect_sessions(traces_dir: &Path) -> Result<Vec<(String, std::time::SystemTime)>> {
    let entries = fs::read_dir(traces_dir)
        .with_context(|| format!("Failed to read traces directory: {}", traces_dir.display()))?;

    let mut sessions = Vec::new();

    for entry in entries {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        if path.is_file() {
            if let Some(extension) = path.extension() {
                if extension == "log" {
                    if let Some(stem) = path.file_stem() {
                        if let Some(session_id) = stem.to_str() {
                            if let Ok(metadata) = entry.metadata() {
                                if let Ok(modified) = metadata.modified() {
                                    sessions.push((session_id.to_string(), modified));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(sessions)
}

fn read_telemetry_log(file_path: &Path) -> Result<Vec<TelemetryLogEntry>> {
    let file = fs::File::open(file_path)
        .with_context(|| format!("Failed to open trace file: {}", file_path.display()))?;

    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line =
            line.with_context(|| format!("Failed to read line {} from trace file", line_num + 1))?;

        if line.trim().is_empty() {
            continue;
        }

        // Skip routing decision JSON from routing engine (not in telemetry format)
        if line.trim().starts_with("{\"stage\":") && !line.contains("\"event\"") {
            continue;
        }

        if let Ok(json) = serde_json::from_str::<Value>(&line) {
            if json.get("event").is_some() {
                if let Ok(entry) = parse_telemetry_entry(&json) {
                    entries.push(entry);
                }
            }
        }
    }

    entries.sort_by_key(|e| e.timestamp);
    Ok(entries)
}

fn parse_telemetry_entry(json: &Value) -> Result<TelemetryLogEntry> {
    let timestamp = json["timestamp"]
        .as_u64()
        .ok_or_else(|| anyhow::anyhow!("Missing or invalid timestamp"))?;

    let severity = json["severity"].as_str().unwrap_or("UNKNOWN").to_string();
    let event = json["event"].as_str().unwrap_or("unknown").to_string();
    let message = json["message"].as_str().unwrap_or("").to_string();

    let attrs = json.get("attributes").and_then(|a| a.as_object());

    let stage = attrs
        .and_then(|a| a.get("stage"))
        .and_then(|s| s.as_str())
        .map(|s| s.to_string());

    let target = attrs
        .and_then(|a| a.get("target"))
        .and_then(|t| t.as_str())
        .map(|t| t.to_string());

    let latency_ms = attrs
        .and_then(|a| a.get("latency_ms"))
        .and_then(|l| l.as_u64())
        .map(|l| l as u32);

    let allowed = attrs
        .and_then(|a| a.get("allowed"))
        .and_then(|a| a.as_bool());

    let reason = attrs
        .and_then(|a| a.get("reason"))
        .and_then(|r| r.as_str())
        .map(|r| r.to_string());

    Ok(TelemetryLogEntry {
        timestamp,
        severity,
        event,
        message,
        stage,
        target,
        latency_ms,
        allowed,
        reason,
    })
}

fn display_telemetry_table(entries: &[TelemetryLogEntry]) {
    println!("{}", "📋 Telemetry Events:".bold().cyan());
    println!("{}", "=".repeat(100).bright_black());

    println!(
        "{:<12} {:<8} {:<20} {:<25} {:<15} {:<20}",
        "Timestamp".bold(),
        "Severity".bold(),
        "Event".bold(),
        "Stage".bold(),
        "Target".bold(),
        "Details".bold()
    );
    println!("{}", "-".repeat(100).bright_black());

    let first_timestamp = entries.first().map(|e| e.timestamp).unwrap_or(0);

    for entry in entries {
        let relative_timestamp = entry.timestamp.saturating_sub(first_timestamp);
        let timestamp_str = format_relative_timestamp(relative_timestamp);

        let (severity_icon, severity_color) = severity_display(&entry.severity);

        let stage = entry.stage.as_deref().unwrap_or("-");
        let target = entry.target.as_deref().unwrap_or("-");

        let target_colored = match target {
            "local" => target.bright_green(),
            "cloud" => target.bright_blue(),
            s if s.starts_with("fallback") => target.bright_yellow(),
            _ => target.white(),
        };

        let details = build_details_string(entry);

        let event_display = truncate(&entry.event, 20);
        let event_colored_display = color_event(&event_display, &entry.event);

        println!(
            "{:<12} {:<8} {:<20} {:<25} {:<15} {:<20}",
            timestamp_str.bright_black(),
            format!("{} {}", severity_icon, severity_color),
            event_colored_display,
            truncate(stage, 25).cyan(),
            target_colored,
            details
        );
    }

    println!("{}", "=".repeat(100).bright_black());
    println!();
}

fn severity_display(severity: &str) -> (&str, colored::ColoredString) {
    match severity {
        "INFO" => ("ℹ️", severity.bright_green()),
        "DEBUG" => ("🔍", severity.bright_blue()),
        "WARN" => ("⚠️", severity.bright_yellow()),
        "ERROR" => ("❌", severity.bright_red()),
        _ => ("•", severity.white()),
    }
}

fn color_event(display: &str, event: &str) -> colored::ColoredString {
    match event {
        "stage_complete" => display.bright_green(),
        "stage_start" => display.bright_cyan(),
        "policy_evaluation" => display.bright_magenta(),
        "routing_decision" => display.bright_blue(),
        "execution_complete" => display.green(),
        "execution_start" => display.cyan(),
        _ => display.white(),
    }
}

fn build_details_string(entry: &TelemetryLogEntry) -> String {
    let mut details = String::new();

    if let Some(latency) = entry.latency_ms {
        let latency_str = format!("{}ms", latency);
        let latency_colored = if latency < 50 {
            latency_str.bright_green()
        } else if latency < 200 {
            latency_str.bright_yellow()
        } else {
            latency_str.bright_red()
        };
        details.push_str(&format!("{} ", latency_colored));
    }
    if let Some(allowed) = entry.allowed {
        let policy_str = format!("Policy: {}", if allowed { "✓" } else { "✗" });
        let policy_colored = if allowed {
            policy_str.bright_green()
        } else {
            policy_str.bright_red()
        };
        details.push_str(&format!("{} ", policy_colored));
    }
    if let Some(ref reason) = entry.reason {
        details.push_str(&format!("Reason: {}", truncate(reason, 30).bright_black()));
    }
    if details.is_empty() {
        details.push_str(&"-".bright_black().to_string());
    }

    details
}

fn display_summary(entries: &[TelemetryLogEntry]) {
    println!("{}", "📊 Summary Statistics:".bold().cyan());
    println!("{}", "=".repeat(60).bright_black());
    println!();

    display_stage_completions(entries);
    display_policy_evaluations(entries);
    display_routing_decisions(entries);

    println!(
        "{} {}",
        "Total Events:".bold(),
        entries.len().to_string().bright_cyan()
    );
}

fn display_stage_completions(entries: &[TelemetryLogEntry]) {
    let stage_completions: Vec<_> = entries
        .iter()
        .filter(|e| e.event == "stage_complete")
        .collect();

    if stage_completions.is_empty() {
        return;
    }

    println!("{}", "Stage Completions:".bold());
    for entry in &stage_completions {
        let stage = entry.stage.as_deref().unwrap_or("unknown");
        let target = entry.target.as_deref().unwrap_or("unknown");
        let latency = entry
            .latency_ms
            .map(|l| format!("{}ms", l))
            .unwrap_or_else(|| "N/A".to_string());

        let target_colored = match target {
            "local" => target.bright_green(),
            "cloud" => target.bright_blue(),
            _ => target.white(),
        };

        let latency_colored = if let Some(lat) = entry.latency_ms {
            if lat < 50 {
                latency.bright_green()
            } else if lat < 200 {
                latency.bright_yellow()
            } else {
                latency.bright_red()
            }
        } else {
            latency.white()
        };

        println!(
            "  {} {} → {} ({})",
            "•".bright_cyan(),
            stage.cyan(),
            target_colored,
            latency_colored
        );
    }
    println!();
}

fn display_policy_evaluations(entries: &[TelemetryLogEntry]) {
    let policy_evals: Vec<_> = entries
        .iter()
        .filter(|e| e.event == "policy_evaluation")
        .collect();

    if policy_evals.is_empty() {
        return;
    }

    let allowed_count = policy_evals
        .iter()
        .filter(|e| e.allowed == Some(true))
        .count();
    let denied_count = policy_evals.len() - allowed_count;

    println!("{}", "Policy Evaluations:".bold());
    println!(
        "  {} Allowed: {}",
        "•".bright_cyan(),
        allowed_count.to_string().bright_green()
    );
    println!(
        "  {} Denied: {}",
        "•".bright_cyan(),
        denied_count.to_string().bright_red()
    );
    println!();
}

fn display_routing_decisions(entries: &[TelemetryLogEntry]) {
    let routing_decisions: Vec<_> = entries
        .iter()
        .filter(|e| e.event == "routing_decision")
        .collect();

    if routing_decisions.is_empty() {
        return;
    }

    println!("{}", "Routing Decisions:".bold());
    for entry in &routing_decisions {
        let stage = entry.stage.as_deref().unwrap_or("unknown");
        let target = entry.target.as_deref().unwrap_or("unknown");
        let reason = entry.reason.as_deref().unwrap_or("N/A");

        let target_colored = match target {
            "local" => target.bright_green(),
            "cloud" => target.bright_blue(),
            _ => target.white(),
        };

        println!(
            "  {} {} → {} ({})",
            "•".bright_cyan(),
            stage.cyan(),
            target_colored,
            truncate(reason, 40).bright_black()
        );
    }
    println!();
}

fn export_trace_summary(entries: &[TelemetryLogEntry], export_path: &Path) -> Result<()> {
    let stage_completions: Vec<_> = entries
        .iter()
        .filter(|e| e.event == "stage_complete")
        .map(|e| {
            json!({
                "stage": e.stage,
                "target": e.target,
                "latency_ms": e.latency_ms,
            })
        })
        .collect();

    let policy_evals: Vec<_> = entries
        .iter()
        .filter(|e| e.event == "policy_evaluation")
        .map(|e| {
            json!({
                "stage": e.stage,
                "allowed": e.allowed,
                "reason": e.reason,
            })
        })
        .collect();

    let routing_decisions: Vec<_> = entries
        .iter()
        .filter(|e| e.event == "routing_decision")
        .map(|e| {
            json!({
                "stage": e.stage,
                "target": e.target,
                "reason": e.reason,
            })
        })
        .collect();

    let severity_counts: HashMap<String, usize> =
        entries.iter().fold(HashMap::new(), |mut acc, e| {
            *acc.entry(e.severity.clone()).or_insert(0) += 1;
            acc
        });

    let event_counts: HashMap<String, usize> = entries.iter().fold(HashMap::new(), |mut acc, e| {
        *acc.entry(e.event.clone()).or_insert(0) += 1;
        acc
    });

    let total_latency: u32 = entries.iter().filter_map(|e| e.latency_ms).sum();

    let summary = json!({
        "session": {
            "total_events": entries.len(),
            "total_latency_ms": total_latency,
            "first_timestamp": entries.first().map(|e| e.timestamp),
            "last_timestamp": entries.last().map(|e| e.timestamp),
        },
        "statistics": {
            "severity_counts": severity_counts,
            "event_counts": event_counts,
            "stage_completions": stage_completions.len(),
            "policy_evaluations": policy_evals.len(),
            "routing_decisions": routing_decisions.len(),
        },
        "stage_completions": stage_completions,
        "policy_evaluations": policy_evals,
        "routing_decisions": routing_decisions,
        "events": entries.iter().map(|e| json!({
            "timestamp": e.timestamp,
            "severity": e.severity,
            "event": e.event,
            "stage": e.stage,
            "target": e.target,
            "latency_ms": e.latency_ms,
            "allowed": e.allowed,
            "reason": e.reason,
        })).collect::<Vec<_>>(),
    });

    let json_str = serde_json::to_string_pretty(&summary)?;
    let mut file = fs::File::create(export_path)
        .with_context(|| format!("Failed to create export file: {}", export_path.display()))?;
    file.write_all(json_str.as_bytes())
        .with_context(|| format!("Failed to write export file: {}", export_path.display()))?;

    Ok(())
}

fn list_sessions(traces_dir: &Path) -> Result<()> {
    if !traces_dir.exists() {
        println!("ℹ️  No traces directory found at: {}", traces_dir.display());
        println!("   Sessions will be created when pipelines are executed.");
        return Ok(());
    }

    let mut sessions = collect_sessions(traces_dir)?;

    if sessions.is_empty() {
        println!("ℹ️  No sessions found.");
        println!("   Sessions will be created when pipelines are executed.");
        return Ok(());
    }

    sessions.sort_by(|a, b| b.1.cmp(&a.1));

    println!("📋 Available Sessions:");
    println!("{}", "=".repeat(60));
    println!();

    for (session_id, modified) in sessions {
        let time_str = format_system_time(modified);
        println!("  • {} (last modified: {})", session_id, time_str);
    }

    println!();
    println!("Usage: xybrid trace --session <session-id>");

    Ok(())
}

// ─── Formatting helpers (trace-specific) ───────────────────────────────────────

/// Format a relative timestamp (ms) as "HH:MM:SS.mmm" or shorter.
fn format_relative_timestamp(ts_ms: u64) -> String {
    let total_secs = ts_ms / 1000;
    let secs = total_secs % 60;
    let mins = (total_secs / 60) % 60;
    let hours = total_secs / 3600;
    let millis = ts_ms % 1000;

    if hours > 0 {
        format!("{:02}:{:02}:{:02}.{:03}", hours, mins, secs, millis)
    } else if mins > 0 {
        format!("{:02}:{:02}.{:03}", mins, secs, millis)
    } else {
        format!("{}.{:03}s", secs, millis)
    }
}

/// Format system time using chrono for human-readable output.
fn format_system_time(time: std::time::SystemTime) -> String {
    use std::time::UNIX_EPOCH;

    if let Ok(duration) = time.duration_since(UNIX_EPOCH) {
        let secs = duration.as_secs();
        let datetime = chrono::DateTime::<chrono::Utc>::from_timestamp(secs as i64, 0);

        if let Some(dt) = datetime {
            dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
        } else {
            "unknown".to_string()
        }
    } else {
        "unknown".to_string()
    }
}

/// Truncate string to max length with ellipsis.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}
