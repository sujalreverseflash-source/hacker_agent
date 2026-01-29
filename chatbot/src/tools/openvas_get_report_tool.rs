use anyhow::Result;
use serde_json::Value;

use crate::services::openvas_get_report;
use crate::Tool;

/// Tool that fetches the final OpenVAS/GVM report by report ID via the Go
/// backend and returns the raw get_reports_response XML.
pub struct OpenVASGetReportTool;

#[async_trait::async_trait]
impl Tool for OpenVASGetReportTool {
    fn name(&self) -> &'static str {
        "openvas_get_report"
    }

    fn description(&self) -> &'static str {
        "Fetches the final OpenVAS/GVM report by report ID via the Go backend."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "report_id": {
                    "type": "string",
                    "description": "OpenVAS report ID whose contents should be fetched."
                }
            },
            "required": ["report_id"],
            "additionalProperties": false
        })
    }

    async fn execute(&self, input: Value) -> Result<Value> {
        let report_id = input
            .get("report_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing required field `report_id`"))?;

        let result = openvas_get_report::openvas_get_report(report_id).await?;
        Ok(result)
    }
}

