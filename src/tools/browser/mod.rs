//! Browser automation module
//!
//! Wraps agent-browser CLI for web automation.

mod executor;
mod snapshot;

pub use executor::BrowserExecutor;
pub use snapshot::{Element, Snapshot};
