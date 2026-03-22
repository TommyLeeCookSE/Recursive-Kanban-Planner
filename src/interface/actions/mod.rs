//! User interface actions and common command handlers.
//!
//! This module coordinates UI events (clicks, drag-and-drop),
//! command feedback, and browser interoperability.
//!
//! For more on how UI actions are structured, see `docs/rust-for-python-devs.md`.

pub mod feedback;
pub mod hooks;
pub mod interop;
pub mod logic;

pub use feedback::*;
pub use hooks::*;
pub use interop::*;
pub use logic::*;
