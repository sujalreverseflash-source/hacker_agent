use anyhow::Result;
use serde_json::Value;

use crate::api::openvas;

/// Business-logic layer for "OpenVAS create task" using the Go backend.
/// This is a thin wrapper around the low-level HTTP client and returns
/// the raw JSON from the Go API, which includes the created task ID
/// under the `id` field and an `existed` flag when a matching task
/// already existed.
pub async fn openvas_create_task(
    name: &str,
    config_id: &str,
    target_id: &str,
) -> Result<Value> {
    openvas::create_task(name, config_id, target_id).await
}

