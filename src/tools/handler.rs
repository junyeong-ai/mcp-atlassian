use crate::config::Config;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait ToolHandler: Send + Sync {
    async fn execute(&self, args: Value, config: &Config) -> Result<Value>;
}
