use anyhow::Result;
use serde_json::{json, Value};

/// Low-level HTTP client for talking to the Go Nmap backend.
pub async fn scan_open_ports(target: &str) -> Result<Value> {
    let client = reqwest::Client::new();
    let resp = client
        .post("http://127.0.0.1:8080/scan-open-ports")
        .json(&json!({ "target": target }))
        .send()
        .await?
        .error_for_status()?;

    let body: Value = resp.json().await?;
    Ok(body)
}

