//! Core module - shared infrastructure for Praxis
//!
//! This module contains foundational types, configuration, and error handling
//! used throughout the application.

pub mod config;
pub mod error;
pub mod types;

pub use config::Config;
pub use error::{PraxisError, Result};
pub use types::*;
