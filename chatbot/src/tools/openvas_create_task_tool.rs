use anyhow::Result;
use serde_json::Value;

use crate::services::openvas_create_task;
use crate::Tool;

/// Tool that creates a new OpenVAS/GVM task via the Go backend
/// and returns the created task ID.
pub struct OpenVASCreateTaskTool;

#[async_trait::async_trait]
impl Tool for OpenVASCreateTaskTool {
    fn name(&self) -> &'static str {
        "openvas_create_task"
    }

    fn description(&self) -> &'static str {
        "Creates an OpenVAS/GVM task (name, config_id, target_id) via the Go backend and returns its ID."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Friendly name for the task."
                },
                "config_id": {
                    "type": "string",
                    "description": "OpenVAS scan configuration ID to use for the task."
                },
                "target_id": {
                    "type": "string",
                    "description": "OpenVAS target ID that this task will scan."
                }
            },
            "required": ["name", "config_id", "target_id"],
            "additionalProperties": false
        })
    }

    async fn execute(&self, input: Value) -> Result<Value> {
        let name = input
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing required field `name`"))?;

        let config_id = input
            .get("config_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing required field `config_id`"))?;

        let target_id = input
            .get("target_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing required field `target_id`"))?;

        let result = openvas_create_task::openvas_create_task(name, config_id, target_id).await?;
        Ok(result)
    }
}

