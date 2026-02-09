
use anyhow::Result;
use serde_json::{json, Value};
use tokio::sync::mpsc;
use futures_util::StreamExt;
use serde::Deserialize;

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

#[derive(Debug, Clone, Deserialize)]
pub struct NmapSSELine {
    pub stream: String, // "stdout" | "stderr"
    pub line: String,
    pub timestamp: String,
}

#[derive(Debug, Clone)]
pub enum NmapStreamEvent {
    Ready(Value),
    Output(NmapSSELine),
    Done(Value),
}

/// Streaming Nmap scan using the Go backend SSE endpoint.
///
/// POST /scan-open-ports/stream
/// Body: same JSON as /scan-open-ports
pub async fn advanced_scan_stream(request_body: &Value) -> Result<mpsc::Receiver<NmapStreamEvent>> {
    let client = reqwest::Client::new();

    let resp = client
        .post("http://127.0.0.1:8080/scan-open-ports/stream")
        .json(request_body)
        .send()
        .await?
        .error_for_status()?;

    let (tx, rx) = mpsc::channel::<NmapStreamEvent>(512);

    tokio::spawn(async move {
        let mut bytes = resp.bytes_stream();

        // Very small SSE parser: accumulates "event:" + "data:" lines until blank line.
        let mut buf = String::new();
        let mut cur_event: Option<String> = None;
        let mut cur_data: Vec<String> = Vec::new();

        while let Some(chunk) = bytes.next().await {
            let Ok(chunk) = chunk else { break };
            let s = String::from_utf8_lossy(&chunk);
            buf.push_str(&s);

            while let Some(pos) = buf.find('\n') {
                let mut line = buf[..pos].to_string();
                buf.drain(..=pos);

                if line.ends_with('\r') {
                    line.pop();
                }

                if line.is_empty() {
                    // dispatch event
                    if let Some(ev) = cur_event.take() {
                        let data = cur_data.join("\n");
                        cur_data.clear();

                        match ev.as_str() {
                            "ready" => {
                                if let Ok(v) = serde_json::from_str::<Value>(&data) {
                                    let _ = tx.send(NmapStreamEvent::Ready(v)).await;
                                }
                            }
                            "output" => {
                                if let Ok(v) = serde_json::from_str::<NmapSSELine>(&data) {
                                    let _ = tx.send(NmapStreamEvent::Output(v)).await;
                                }
                            }
                            "done" => {
                                if let Ok(v) = serde_json::from_str::<Value>(&data) {
                                    let _ = tx.send(NmapStreamEvent::Done(v)).await;
                                }
                                return;
                            }
                            _ => {
                                // ignore unknown events
                            }
                        }
                    } else {
                        cur_data.clear();
                    }
                    continue;
                }

                if let Some(rest) = line.strip_prefix("event:") {
                    cur_event = Some(rest.trim().to_string());
                    continue;
                }
                if let Some(rest) = line.strip_prefix("data:") {
                    cur_data.push(rest.trim_start().to_string());
                    continue;
                }
                // ignore comments/other fields
            }
        }
    });

    Ok(rx)
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

pub async fn scan_open_ports_stream(target: &str, timing: Option<&str>) -> Result<mpsc::Receiver<NmapStreamEvent>> {
    let mut body = json!({
        "target": target
    });
    if let Some(t) = timing {
        body["timing"] = json!(t);
    }
    advanced_scan_stream(&body).await
}
