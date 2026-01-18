use anyhow::Result;
use serde_json::Value;

/// Low-level HTTP client for talking to the Go OpenVAS backend.
/// Currently only exposes the "get version" API.
pub async fn get_version() -> Result<Value> {
    let client = reqwest::Client::new();
    let resp = client
        .get("http://127.0.0.1:8080/openvas/version")
        .send()
        .await?
        .error_for_status()?;

    let body: Value = resp.json().await?;
    Ok(body)
}

