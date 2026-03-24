#![allow(non_snake_case)]

//! # Interface Layer
//!
//! This module houses the UI layer of the application using Dioxus.
//! It includes the root application component, individual UI components,
//! routing definitions, and error UI templates.

pub mod actions;
pub mod app;
#[macro_use]
pub mod components;
pub mod error_templates;
pub mod routes;

pub use app::App;
pub use routes::Route;
