use anyhow::Result;
use serde_json::Value;

use crate::services::openvas_get_version;
use crate::Tool;

/// Tool that fetches the OpenVAS/GVM version via the Go backend.
pub struct OpenVASGetVersionTool;

#[async_trait::async_trait]
impl Tool for OpenVASGetVersionTool {
    fn name(&self) -> &'static str {
        "openvas_get_version"
    }

    fn description(&self) -> &'static str {
        "Fetches the OpenVAS/GVM version via the Go backend."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "description": "No input fields required."
        })
    }

    async fn execute(&self, _input: Value) -> Result<Value> {
        openvas_get_version::openvas_get_version().await
    }
}

