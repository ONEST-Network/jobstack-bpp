use chrono::{Duration, NaiveDate, Utc};
use std::fs;
use std::path::Path;
use tracing::error;
use tracing::info;
use tracing_appender::{non_blocking::WorkerGuard, rolling};
use tracing_subscriber::{fmt, fmt::time::UtcTime, prelude::*, EnvFilter};

fn cleanup_old_logs(log_dir: &str, service_name: &str, days_to_keep: u64) {
    let path = Path::new(log_dir);
    if !path.exists() {
        return;
    }

    let today = Utc::now().date_naive();
    let cutoff_date = today - Duration::days(days_to_keep as i64);

    match fs::read_dir(path) {
        Ok(entries) => {
            for entry in entries.filter_map(Result::ok) {
                let file_path = entry.path();
                if file_path.is_file() {
                    if let Some(file_name) = file_path.file_name().and_then(|n| n.to_str()) {
                        // Expected format: service_name.log.YYYY-MM-DD
                        let prefix = format!("{}.log.", service_name);
                        if file_name.starts_with(&prefix) {
                            let date_str = &file_name[prefix.len()..];
                            if let Ok(file_date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                                if file_date < cutoff_date {
                                    if let Err(e) = fs::remove_file(&file_path) {
                                        error!(
                                            "Failed to delete old log file {:?}: {}",
                                            file_path, e
                                        );
                                    } else {
                                        info!("Deleted old log file: {:?}", file_path);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(e) => error!("Failed to read log directory {:?}: {}", path, e),
    }
}

pub fn setup_logging(log_dir: &str, svc: &str, log_retention_days: u64) -> WorkerGuard {
    cleanup_old_logs(log_dir, svc, log_retention_days);

    let log_file_name = format!("{}.log", svc);

    let (file_writer, file_guard) =
        tracing_appender::non_blocking(rolling::daily(log_dir, log_file_name));

    let file_layer = fmt::layer()
        .with_writer(file_writer)
        .json()
        .with_timer(UtcTime::rfc_3339())
        .with_target(true)
        .with_thread_ids(false)
        .with_filter(EnvFilter::new("info"));

    let console_layer = fmt::layer()
        .compact()
        .with_timer(UtcTime::rfc_3339())
        .with_target(true)
        .with_thread_ids(false)
        .with_filter(EnvFilter::new("info"));

    tracing::subscriber::set_global_default(
        tracing_subscriber::registry()
            .with(file_layer)
            .with(console_layer),
    )
    .expect("Failed to set global subscriber");

    file_guard
}

pub fn log_cron_job(icon: &str, message: &str) {
    let content = format!("{} {}", icon, message);

    let width = 44;
    let padded = format!("{:<width$}", content, width = width - 2);

    info!(target: "cron", "╔{}╗", "═".repeat(width));
    info!(target: "cron", "║ {} ║", padded);
    info!(target: "cron", "╚{}╝", "═".repeat(width));
}
