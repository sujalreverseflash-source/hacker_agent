use anyhow::Result;
use serde_json::Value;

use crate::services::nmap_normal_scan;
use crate::Tool;

/// Tool that exposes a "normal" Nmap open-port scan via the Go backend.
pub struct NmapOpenPortsTool;

#[async_trait::async_trait]
impl Tool for NmapOpenPortsTool {
    fn name(&self) -> &'static str {
        "nmap_open_ports"
    }

    fn description(&self) -> &'static str {
        "Scans open TCP ports on a given target (IP or hostname) via the Go backend."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "target": {
                    "type": "string",
                    "description": "Target hostname or IP address to scan."
                }
            },
            "required": ["target"],
            "additionalProperties": false
        })
    }

    async fn execute(&self, input: Value) -> Result<Value> {
        let target = input
            .get("target")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing required field `target`"))?;

        nmap_normal_scan::nmap_normal_scan(target).await
    }
}

