use std::collections::VecDeque;
use std::sync::{LazyLock, Mutex};
use thiserror::Error;
use tracing::Level;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

const DIAGNOSTIC_CAPACITY: usize = 1000;

static DIAGNOSTICS: LazyLock<Mutex<VecDeque<DiagnosticEntry>>> =
    LazyLock::new(|| Mutex::new(VecDeque::with_capacity(DIAGNOSTIC_CAPACITY)));

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticEntry {
    pub timestamp_unix_secs: u64,
    pub level: String,
    pub target: String,
    pub message: String,
}

pub struct LoggingGuard {
    #[cfg(not(target_arch = "wasm32"))]
    _worker_guard: tracing_appender::non_blocking::WorkerGuard,
}

#[derive(Debug, Error)]
pub enum LoggingInitError {
    #[cfg(not(target_arch = "wasm32"))]
    #[error("Failed to create runtime log directory: {0}")]
    CreateLogDir(#[source] std::io::Error),

    #[cfg(not(target_arch = "wasm32"))]
    #[error("Failed to resolve the current working directory: {0}")]
    ResolveCurrentDir(#[source] std::io::Error),

    #[error("Failed to initialize tracing subscriber: {0}")]
    Subscriber(#[from] tracing_subscriber::util::TryInitError),
}

pub fn init_logging() -> Result<LoggingGuard, LoggingInitError> {
    #[cfg(target_arch = "wasm32")]
    {
        console_error_panic_hook::set_once();

        let subscriber = tracing_wasm::WASMLayer::new(tracing_wasm::WASMLayerConfig::default());
        tracing_subscriber::registry().with(subscriber).try_init()?;

        tracing::info!(
            version = env!("CARGO_PKG_VERSION"),
            target = target_name(),
            feature = feature_name(),
            log_level = resolved_log_level(),
            "Logging initialized for web runtime"
        );
        record_diagnostic(Level::INFO, "startup", "Web logging initialized");

        Ok(LoggingGuard {})
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::backtrace::Backtrace;
        use std::env;
        use std::fs;
        use tracing_subscriber::EnvFilter;

        let current_dir = env::current_dir().map_err(LoggingInitError::ResolveCurrentDir)?;
        let runtime_log_dir = current_dir.join("logs").join("runtime");
        fs::create_dir_all(&runtime_log_dir).map_err(LoggingInitError::CreateLogDir)?;

        let file_appender = tracing_appender::rolling::daily(runtime_log_dir, "kanban-planner.log");
        let (file_writer, worker_guard) = tracing_appender::non_blocking(file_appender);

        let log_level = resolved_log_level();
        let file_filter = EnvFilter::try_new(&log_level).unwrap_or_else(|_| EnvFilter::new("info"));
        let stderr_filter =
            EnvFilter::try_new(&log_level).unwrap_or_else(|_| EnvFilter::new("info"));

        let file_layer = tracing_subscriber::fmt::layer()
            .with_ansi(false)
            .with_writer(file_writer)
            .with_target(true)
            .with_filter(file_filter);

        let stderr_layer = tracing_subscriber::fmt::layer()
            .with_writer(std::io::stderr)
            .with_target(true)
            .with_filter(stderr_filter);

        tracing_subscriber::registry()
            .with(file_layer)
            .with(stderr_layer)
            .try_init()?;

        let previous_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            let backtrace = Backtrace::force_capture();
            tracing::error!(
                panic = %panic_info,
                backtrace = %backtrace,
                "Unhandled panic captured"
            );
            record_diagnostic(
                Level::ERROR,
                "panic",
                format!("Unhandled panic: {panic_info}"),
            );
            previous_hook(panic_info);
        }));

        tracing::info!(
            version = env!("CARGO_PKG_VERSION"),
            target = target_name(),
            feature = feature_name(),
            cwd = %current_dir.display(),
            log_level = %log_level,
            "Logging initialized for native runtime"
        );
        record_diagnostic(
            Level::INFO,
            "startup",
            format!("Native logging initialized in {}", current_dir.display()),
        );

        Ok(LoggingGuard {
            _worker_guard: worker_guard,
        })
    }
}

pub fn diagnostics_snapshot() -> Vec<DiagnosticEntry> {
    DIAGNOSTICS
        .lock()
        .expect("diagnostics lock poisoned")
        .iter()
        .cloned()
        .collect()
}

pub fn record_diagnostic(level: Level, target: &str, message: impl Into<String>) {
    let entry = DiagnosticEntry {
        timestamp_unix_secs: unix_timestamp_secs(),
        level: level.as_str().to_string(),
        target: target.to_string(),
        message: message.into(),
    };

    let mut diagnostics = DIAGNOSTICS.lock().expect("diagnostics lock poisoned");
    if diagnostics.len() >= DIAGNOSTIC_CAPACITY {
        diagnostics.pop_front();
    }
    diagnostics.push_back(entry);
}

pub fn feature_name() -> &'static str {
    #[cfg(feature = "desktop")]
    {
        "desktop"
    }
    #[cfg(all(not(feature = "desktop"), feature = "mobile"))]
    {
        "mobile"
    }
    #[cfg(all(not(feature = "desktop"), not(feature = "mobile"), feature = "web"))]
    {
        "web"
    }
    #[cfg(all(
        not(feature = "desktop"),
        not(feature = "mobile"),
        not(feature = "web")
    ))]
    {
        "unknown"
    }
}

pub fn target_name() -> &'static str {
    #[cfg(target_arch = "wasm32")]
    {
        "wasm32"
    }
    #[cfg(all(not(target_arch = "wasm32"), target_os = "windows"))]
    {
        "windows"
    }
    #[cfg(all(not(target_arch = "wasm32"), target_os = "macos"))]
    {
        "macos"
    }
    #[cfg(all(not(target_arch = "wasm32"), target_os = "linux"))]
    {
        "linux"
    }
    #[cfg(all(
        not(target_arch = "wasm32"),
        not(target_os = "windows"),
        not(target_os = "macos"),
        not(target_os = "linux")
    ))]
    {
        "unknown"
    }
}

fn resolved_log_level() -> String {
    std::env::var("KANBAN_LOG_LEVEL").unwrap_or_else(|_| "info".to_string())
}

fn unix_timestamp_secs() -> u64 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or_default()
    }
    #[cfg(target_arch = "wasm32")]
    {
        // SystemTime is not available on wasm32; return 0 as a safe fallback.
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostics_ring_buffer_keeps_latest_entries() {
        DIAGNOSTICS
            .lock()
            .expect("diagnostics lock poisoned")
            .clear();
        let target = "logging-test";

        for index in 0..(DIAGNOSTIC_CAPACITY + 5) {
            record_diagnostic(Level::INFO, target, format!("entry-{index}"));
        }

        let snapshot: Vec<_> = diagnostics_snapshot()
            .into_iter()
            .filter(|entry| entry.target == target)
            .collect();
        assert!(snapshot.len() <= DIAGNOSTIC_CAPACITY);
        assert!(
            !snapshot.iter().any(|entry| entry.message == "entry-0"),
            "oldest entries should be rotated out"
        );
        assert_eq!(
            snapshot
                .first()
                .map(|entry| entry.message.starts_with("entry-")),
            Some(true)
        );
        assert_eq!(
            snapshot.last().map(|entry| entry.message.as_str()),
            Some("entry-1004")
        );
    }
}
