// tests/common/mod.rs
// Shared test utilities module

#![allow(dead_code)]
#![allow(unused_imports)]

pub mod fixtures;
pub mod mocks;

// Re-export commonly used test utilities
pub use fixtures::{ConfigBuilder, ConfluenceFixtures, JiraFixtures};
pub use mocks::{MockAtlassianServer, setup_mock_server};
