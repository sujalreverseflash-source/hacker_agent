use anyhow::Result;
use serde_json::Value;

use crate::api::openvas;

/// Business-logic layer for "OpenVAS create target" using the Go backend.
/// For now this is a thin wrapper around the low-level HTTP client.
/// It returns the raw JSON from the Go API, which includes the created
/// target ID under the `id` field.
pub async fn openvas_create_target(
    name: &str,
    hosts: &str,
    port_range: Option<&str>,
) -> Result<Value> {
    openvas::create_target(name, hosts, port_range).await
}

