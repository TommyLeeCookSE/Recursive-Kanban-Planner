use kanban_planner::App;
use kanban_planner::infrastructure::logging::{
    feature_name, init_logging, record_diagnostic, target_name,
};
use tracing::{Level, info};

fn main() {
    let _logging_guard = match init_logging() {
        Ok(guard) => Some(guard),
        Err(error) => {
            eprintln!("Failed to initialize logging: {error}");
            None
        }
    };

    let current_dir = std::env::current_dir()
        .map(|dir| dir.display().to_string())
        .unwrap_or_else(|_| "<unavailable>".to_string());
    let backtrace = std::env::var("RUST_BACKTRACE").unwrap_or_else(|_| "unset".to_string());
    let log_level = std::env::var("KANBAN_LOG_LEVEL").unwrap_or_else(|_| "info".to_string());

    info!(
        version = env!("CARGO_PKG_VERSION"),
        feature = feature_name(),
        target = target_name(),
        cwd = %current_dir,
        rust_backtrace = %backtrace,
        log_level = %log_level,
        "Launching Dioxus application"
    );
    record_diagnostic(Level::INFO, "startup", "Launching Dioxus application");

    // Launch the Dioxus app. The `dx` CLI determines the platform at compile time.
    dioxus::launch(App);

    info!("Dioxus application shutdown complete");
    record_diagnostic(
        Level::INFO,
        "shutdown",
        "Dioxus application shutdown complete",
    );
}
