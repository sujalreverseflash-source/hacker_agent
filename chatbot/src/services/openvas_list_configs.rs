use anyhow::Result;
use serde_json::Value;

use crate::api::openvas;

/// Business-logic layer for "OpenVAS list configs" using the Go backend.
/// Right now this is just a thin wrapper returning the backend JSON as-is.
pub async fn openvas_list_configs() -> Result<Value> {
    openvas::list_configs().await
}

