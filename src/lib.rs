//! # Recursive Kanban Planner
//!
//! A clean-architecture based Kanban board where every card is itself a board,
//! allowing for infinite structural depth.

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod interface;

// Binary entry point will use App from the interface layer.
pub use interface::App;
