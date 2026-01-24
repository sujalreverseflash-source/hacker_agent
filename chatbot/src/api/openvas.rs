use anyhow::Result;
use serde_json::Value;

/// Low-level HTTP client for talking to the Go OpenVAS backend.
/// Currently exposes:
///  - "get version"
///  - "list configs"
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

/// Fetch all available OpenVAS scan configurations (profiles) from the Go backend.
/// The Go API returns a JSON object of the form:
/// {
///   "configs": [
///     { "id": "...", "name": "...", "comment": "..." },
///     ...
///   ]
/// }
pub async fn list_configs() -> Result<Value> {
    let client = reqwest::Client::new();
    let resp = client
        .get("http://127.0.0.1:8080/openvas/configs")
        .send()
        .await?
        .error_for_status()?;

    let body: Value = resp.json().await?;
    Ok(body)
}


