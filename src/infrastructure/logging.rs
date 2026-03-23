//! Runtime logging and diagnostic capture for the Kanban Planner.
//!
//! This module provides the logging foundation for both native and web targets,
//! including an in-memory diagnostic ring buffer for UI-level inspection.
//!
//! For a comparison of Rust logging vs Python logging, see
//! `docs/rust-for-python-devs.md`.

use serde::Deserialize;
use std::collections::{HashMap, VecDeque};
use std::sync::{LazyLock, Mutex};
use thiserror::Error;
use tracing::Level;
#[cfg(not(target_arch = "wasm32"))]
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

const DEFAULT_DIAGNOSTIC_CAPACITY: usize = 1000;

static DIAGNOSTICS: LazyLock<Mutex<VecDeque<DiagnosticEntry>>> =
    LazyLock::new(|| Mutex::new(VecDeque::with_capacity(DEFAULT_DIAGNOSTIC_CAPACITY)));

static DIAGNOSTIC_CAPACITY: LazyLock<Mutex<usize>> =
    LazyLock::new(|| Mutex::new(DEFAULT_DIAGNOSTIC_CAPACITY));

/// A single in-memory diagnostic entry captured for later inspection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticEntry {
    /// Unix timestamp in seconds when the entry was recorded.
    pub timestamp_unix_secs: u64,
    /// The string representation of the log level (e.g., "INFO", "ERROR").
    pub level: String,
    /// The log target, typically the module path.
    pub target: String,
    /// The actual log message content.
    pub message: String,
}

/// Dynamic logging configuration fetched at startup.
#[derive(Clone, Debug, Deserialize, Default)]
pub struct LoggingConfig {
    /// Global log level (e.g., "info", "debug").
    pub global_level: Option<String>,
    /// Module-specific level overrides.
    pub overrides: Option<HashMap<String, String>>,
    /// Max entries in the in-memory log buffer.
    pub max_buffer_capacity: Option<usize>,
}

impl LoggingConfig {
    /// Returns a tracing-compatible EnvFilter string based on this config.
    pub fn to_filter_string(&self) -> String {
        let mut filter = self
            .global_level
            .clone()
            .unwrap_or_else(|| "info".to_string());
        if let Some(overrides) = &self.overrides {
            for (module, level) in overrides {
                filter.push_str(&format!(",{module}={level}"));
            }
        }
        filter
    }
}

/// Keeps native logging resources alive for the life of the application.
pub struct LoggingGuard {
    #[cfg(not(target_arch = "wasm32"))]
    _worker_guard: tracing_appender::non_blocking::WorkerGuard,
}

/// Errors that can occur while initializing the logging subsystem.
#[derive(Debug, Error)]
pub enum LoggingInitError {
    /// Failed to create the directory for runtime logs on disk.
    #[cfg(not(target_arch = "wasm32"))]
    #[error("Failed to create runtime log directory: {0}")]
    CreateLogDir(#[source] std::io::Error),

    /// Failed to determine the current working directory.
    #[cfg(not(target_arch = "wasm32"))]
    #[error("Failed to resolve the current working directory: {0}")]
    ResolveCurrentDir(#[source] std::io::Error),

    /// Failed to fetch or parse the external configuration.
    #[error("Failed to load external logging configuration: {0}")]
    Config(String),

    /// Failed to install the tracing subscriber globally.
    #[error("Failed to initialize tracing subscriber: {0}")]
    Subscriber(#[from] tracing_subscriber::util::TryInitError),
}

/// Fetches the logging configuration from the server (Web only).
#[cfg(target_arch = "wasm32")]
pub async fn fetch_config() -> Result<LoggingConfig, LoggingInitError> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Request, RequestInit, RequestMode, Response};

    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::Cors);

    let url = "/assets/logging.toml";
    let request = Request::new_with_str_and_init(url, &opts)
        .map_err(|e| LoggingInitError::Config(format!("Failed to create request: {e:?}")))?;

    let window =
        web_sys::window().ok_or_else(|| LoggingInitError::Config("No window found".into()))?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| LoggingInitError::Config(format!("Fetch error: {e:?}")))?;

    let resp: Response = resp_value.dyn_into().unwrap();
    if !resp.ok() {
        return Err(LoggingInitError::Config(format!(
            "HTTP error: {}",
            resp.status()
        )));
    }

    let text_value = JsFuture::from(
        resp.text()
            .map_err(|e| LoggingInitError::Config(format!("Text conversion error: {e:?}")))?,
    )
    .await
    .map_err(|e| LoggingInitError::Config(format!("Failed to read response body: {e:?}")))?;

    let toml_text = text_value
        .as_string()
        .ok_or_else(|| LoggingInitError::Config("Response body is not a string".into()))?;
    toml::from_str(&toml_text)
        .map_err(|e| LoggingInitError::Config(format!("TOML parse error: {e}")))
}

/// A custom tracing layer that captures all events into the in-memory diagnostics buffer.
struct DiagnosticLayer;

impl<S> tracing_subscriber::Layer<S> for DiagnosticLayer
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let metadata = event.metadata();
        let level = *metadata.level();
        let target = metadata.target();

        // Extract the message from the event fields
        let mut visitor = MessageVisitor::default();
        event.record(&mut visitor);
        let message = visitor.0;

        record_diagnostic(level, target, message);
    }
}

#[derive(Default)]
struct MessageVisitor(String);

impl tracing::field::Visit for MessageVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.0 = format!("{value:?}");
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.0 = value.to_string();
        }
    }
}

/// Initializes runtime logging for the web target.
#[cfg(target_arch = "wasm32")]
pub async fn init_logging() -> Result<LoggingGuard, LoggingInitError> {
    console_error_panic_hook::set_once();

    let config = fetch_config().await.unwrap_or_default();
    if let Some(capacity) = config.max_buffer_capacity {
        let mut cap = DIAGNOSTIC_CAPACITY.lock().unwrap();
        *cap = capacity;
        let mut diagnostics = DIAGNOSTICS.lock().unwrap();
        let current_len = diagnostics.len();
        diagnostics.reserve_exact(capacity.saturating_sub(current_len));
    }

    let subscriber = tracing_wasm::WASMLayer::new(tracing_wasm::WASMLayerConfig::default());
    let filter_str = config.to_filter_string();
    let filter = tracing_subscriber::EnvFilter::try_new(&filter_str)
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("kanban_planner=info,warn"));

    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(subscriber)
        .with(DiagnosticLayer)
        .try_init();

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        target = target_name(),
        feature = feature_name(),
        log_level = %filter_str,
        "Logging initialized for web runtime"
    );

    Ok(LoggingGuard {})
}

/// Initializes runtime logging for native targets.
#[cfg(not(target_arch = "wasm32"))]
pub fn init_logging() -> Result<LoggingGuard, LoggingInitError> {
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
    let stderr_filter = EnvFilter::try_new(&log_level).unwrap_or_else(|_| EnvFilter::new("info"));

    let file_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_writer(file_writer)
        .with_target(true)
        .with_filter(file_filter);

    let stderr_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stderr)
        .with_target(true)
        .with_filter(stderr_filter);

    let _ = tracing_subscriber::registry()
        .with(file_layer)
        .with(stderr_layer)
        .with(DiagnosticLayer)
        .try_init();

    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let backtrace = Backtrace::force_capture();
        tracing::error!(
            panic = %panic_info,
            backtrace = %backtrace,
            "Unhandled panic captured"
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

    Ok(LoggingGuard {
        _worker_guard: worker_guard,
    })
}

/// Returns a snapshot of the in-memory diagnostics ring buffer.
pub fn diagnostics_snapshot() -> Vec<DiagnosticEntry> {
    DIAGNOSTICS
        .lock()
        .expect("diagnostics lock poisoned")
        .iter()
        .cloned()
        .collect()
}

/// Records a diagnostic entry in the in-memory ring buffer.
pub fn record_diagnostic(level: Level, target: &str, message: impl Into<String>) {
    let entry = DiagnosticEntry {
        timestamp_unix_secs: unix_timestamp_secs(),
        level: level.as_str().to_string(),
        target: target.to_string(),
        message: message.into(),
    };

    let capacity = *DIAGNOSTIC_CAPACITY.lock().expect("capacity lock poisoned");
    let mut diagnostics = DIAGNOSTICS.lock().expect("diagnostics lock poisoned");
    if diagnostics.len() >= capacity {
        diagnostics.pop_front();
    }
    diagnostics.push_back(entry);
}

/// Returns the enabled app feature name for the current build.
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

/// Returns a simplified runtime target label for diagnostics and startup logs.
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

#[cfg(not(target_arch = "wasm32"))]
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
        // SystemTime is unavailable on wasm32; returning 0 as a placeholder.
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
        let capacity = *DIAGNOSTIC_CAPACITY.lock().expect("capacity lock poisoned");

        for index in 0..(capacity + 5) {
            record_diagnostic(Level::INFO, target, format!("entry-{index}"));
        }

        let snapshot: Vec<_> = diagnostics_snapshot()
            .into_iter()
            .filter(|entry| entry.target == target)
            .collect();
        assert!(snapshot.len() <= capacity);
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
        let expected_last = format!("entry-{}", capacity + 4);
        assert_eq!(
            snapshot.last().map(|entry| entry.message.as_str()),
            Some(expected_last.as_str())
        );
    }
}
