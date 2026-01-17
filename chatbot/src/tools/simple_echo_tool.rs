use anyhow::Result;
use serde_json::Value;

use crate::Tool;

/// Simple echo tool used mainly for testing the MCP plumbing.
pub struct EchoTool;

#[async_trait::async_trait]
impl Tool for EchoTool {
    fn name(&self) -> &'static str {
        "echo"
    }

    fn description(&self) -> &'static str {
        "Echoes back the given JSON input."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "description": "Any JSON object to echo back."
        })
    }

    async fn execute(&self, input: Value) -> Result<Value> {
        Ok(serde_json::json!({ "echo": input }))
    }
}

