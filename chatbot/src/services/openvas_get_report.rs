use anyhow::Result;
use serde_json::Value;

use crate::api::openvas;

/// Business-logic layer for "OpenVAS get report" using the Go backend.
/// Thin wrapper around the low-level HTTP client. Returns the raw JSON
/// from the Go API, which includes the `report_id` and `response_raw`
/// (the XML <get_reports_response/> from gvmd).
pub async fn openvas_get_report(report_id: &str) -> Result<Value> {
    openvas::get_report(report_id).await
}

