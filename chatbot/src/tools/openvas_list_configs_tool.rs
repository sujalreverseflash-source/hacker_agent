use anyhow::Result;
use serde_json::Value;

use crate::services::openvas_list_configs;
use crate::Tool;

/// Tool that lists all available OpenVAS/GVM scan configurations via the Go backend.
pub struct OpenVASListConfigsTool;

#[async_trait::async_trait]
impl Tool for OpenVASListConfigsTool {
    fn name(&self) -> &'static str {
        "openvas_list_scan_configs"
    }

    fn description(&self) -> &'static str {
        "Lists all available OpenVAS/GVM scan configurations (profiles) via the Go backend."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "description": "No input fields required."
        })
    }

    async fn execute(&self, _input: Value) -> Result<Value> {
        openvas_list_configs::openvas_list_configs().await
    }
}

