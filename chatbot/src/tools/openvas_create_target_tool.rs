use anyhow::Result;
use serde_json::Value;

use crate::services::openvas_create_target;
use crate::Tool;

/// Tool that creates a new OpenVAS/GVM target via the Go backend
/// and returns the created target ID.
pub struct OpenVASCreateTargetTool;

#[async_trait::async_trait]
impl Tool for OpenVASCreateTargetTool {
    fn name(&self) -> &'static str {
        "openvas_create_target"
    }

    fn description(&self) -> &'static str {
        "Creates an OpenVAS/GVM target (name, hosts, optional port_range) via the Go backend and returns its ID."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Friendly name for the target."
                },
                "hosts": {
                    "type": "string",
                    "description": "Hostname/IP or CIDR understood by OpenVAS."
                },
                "port_range": {
                    "type": "string",
                    "description": "Optional port range string (e.g. '1-65535' or '62078')."
                }
            },
            "required": ["name", "hosts"],
            "additionalProperties": false
        })
    }

    async fn execute(&self, input: Value) -> Result<Value> {
        let name = input
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing required field `name`"))?;

        let hosts = input
            .get("hosts")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing required field `hosts`"))?;

        let port_range = input
            .get("port_range")
            .and_then(|v| v.as_str());

        let result = openvas_create_target::openvas_create_target(name, hosts, port_range).await?;
        Ok(result)
    }
}

