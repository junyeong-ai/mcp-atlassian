use serde_json::Value;
use anyhow::Result;
use async_trait::async_trait;
use crate::config::Config;

#[async_trait]
pub trait ToolHandler: Send + Sync {
    async fn execute(&self, args: Value, config: &Config) -> Result<Value>;
}