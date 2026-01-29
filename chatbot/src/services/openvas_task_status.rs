use anyhow::Result;
use serde_json::Value;

use crate::api::openvas;

/// Business-logic layer for "OpenVAS task status" using the Go backend.
/// Thin wrapper around the low-level HTTP client. Returns the raw JSON
/// from the Go API, which includes the `task_id` and `response_raw`
/// (the XML <get_tasks_response/> from gvmd).
pub async fn openvas_task_status(task_id: &str) -> Result<Value> {
    openvas::get_task_status(task_id).await
}

