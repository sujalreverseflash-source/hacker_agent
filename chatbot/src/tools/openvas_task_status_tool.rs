use anyhow::Result;
use serde_json::Value;

use crate::services::openvas_task_status;
use crate::Tool;

/// Tool that fetches the current status/details for an existing OpenVAS/GVM
/// task via the Go backend and returns the raw get_tasks_response XML.
pub struct OpenVASTaskStatusTool;

#[async_trait::async_trait]
impl Tool for OpenVASTaskStatusTool {
    fn name(&self) -> &'static str {
        "openvas_task_status"
    }

    fn description(&self) -> &'static str {
        "Fetches the current status/details for an existing OpenVAS/GVM task by ID via the Go backend."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "task_id": {
                    "type": "string",
                    "description": "OpenVAS task ID whose status should be fetched."
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

        let result = openvas_task_status::openvas_task_status(task_id).await?;
        Ok(result)
    }
}

