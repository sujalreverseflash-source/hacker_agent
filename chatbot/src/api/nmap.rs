
use anyhow::Result;
use serde_json::{json, Value};

/// Advanced Nmap scan with comprehensive options
pub async fn advanced_scan(request_body: &Value) -> Result<Value> {
    let client = reqwest::Client::new();
    
    let resp = client
        .post("http://127.0.0.1:8080/scan-open-ports")
        .json(request_body)
        .send()
        .await?
        .error_for_status()?;

    let response_body: Value = resp.json().await?;
    Ok(response_body)
}

/// Legacy simple scan for backward compatibility
pub async fn scan_open_ports(target: &str, timing: Option<&str>) -> Result<Value> {
    let mut body = json!({
        "target": target
    });
    
    if let Some(t) = timing {
        body["timing"] = json!(t);
    }
    
    advanced_scan(&body).await
}
