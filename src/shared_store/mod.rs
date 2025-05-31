//! Shared store implementations for PocketFlow
//!
//! This module provides both synchronous and asynchronous shared store implementations
//! for data communication between nodes in PocketFlow workflows.

pub mod async_store;
pub mod sync;

// Re-export the main types for convenience
pub use async_store::AsyncSharedStore;
pub use sync::{InMemorySharedStore, SharedStore};

#[cfg(test)]
mod tests {
    // Module-level integration tests can go here
}
