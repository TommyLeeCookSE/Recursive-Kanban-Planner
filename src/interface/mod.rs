//! # Interface Layer
//!
//! This module houses the UI layer of the application using Dioxus.
//! It includes the root application component, individual UI components,
//! routing definitions, and error UI templates.

pub mod app;
pub mod components;
pub mod error_templates;
pub mod routes;

// Re-export the root App component for convenience at the binary layer.
pub use app::App;
