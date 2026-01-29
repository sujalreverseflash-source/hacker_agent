use anyhow::Result;
use serde_json::Value;

use crate::services::openvas_start_task;
use crate::Tool;

/// Tool that starts an existing OpenVAS/GVM task via the Go backend
/// and returns the raw start_task_response XML.
pub struct OpenVASStartTaskTool;

#[async_trait::async_trait]
impl Tool for OpenVASStartTaskTool {
    fn name(&self) -> &'static str {
        "openvas_start_task"
    }

    fn description(&self) -> &'static str {
        "Starts an existing OpenVAS/GVM task by ID via the Go backend and returns the raw XML response."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "task_id": {
                    "type": "string",
                    "description": "OpenVAS task ID to start."
                }
            },
            "required": ["task_id"],
            "additionalProperties": false
        })
    }

    async fn execute(&self, input: Value) -> Result<Value> {
        let task_id = input
            .get("task_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing required field `task_id`"))?;

        let result = openvas_start_task::openvas_start_task(task_id).await?;
        Ok(result)
    }
}

