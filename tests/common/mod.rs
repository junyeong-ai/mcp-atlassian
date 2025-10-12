// tests/common/mod.rs
// Shared test utilities module

pub mod fixtures;
pub mod mocks;

// Re-export commonly used test utilities
pub use fixtures::{ConfigBuilder, JiraFixtures, ConfluenceFixtures};
pub use mocks::{MockAtlassianServer, setup_mock_server};
