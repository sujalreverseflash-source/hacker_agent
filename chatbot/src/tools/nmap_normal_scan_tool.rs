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
        "Scans open TCP ports on a given target with optional timing template (T0-T5)."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "target": {
                    "type": "string",
                    "description": "Target hostname or IP address to scan."
                },
                "timing": {
                    "type": "string",
                    "description": "Nmap timing template: T0 (Paranoid), T1 (Sneaky), T2 (Polite), T3 (Normal), T4 (Aggressive), T5 (Insane). Default: T2",
                    "enum": ["T0", "T1", "T2", "T3", "T4", "T5"]
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
            
        let timing = input
            .get("timing")
            .and_then(|v| v.as_str());

        nmap_normal_scan::nmap_normal_scan(target, timing).await
    }
}

