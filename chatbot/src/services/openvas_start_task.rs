use anyhow::Result;
use serde_json::Value;

use crate::api::openvas;

/// Business-logic layer for "OpenVAS start task" using the Go backend.
/// Thin wrapper around the low-level HTTP client. Returns the raw JSON
/// from the Go API, which includes the `task_id` and `response_raw`
/// (the XML <start_task_response/> from gvmd).
pub async fn openvas_start_task(task_id: &str) -> Result<Value> {
    openvas::start_task(task_id).await
}

