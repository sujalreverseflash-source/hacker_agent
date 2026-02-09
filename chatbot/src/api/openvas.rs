use anyhow::Result;
use serde_json::{Map, Value};
use tokio::sync::mpsc;
use futures_util::StreamExt;

/// Low-level HTTP client for talking to the Go OpenVAS backend.
/// Currently exposes:
///  - "get version"
///  - "list configs"
///  - "create target"
///  - "create task"
///  - "start task"
///  - "get task status"
///  - "get report"
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

/// Create (or reuse) an OpenVAS target via the Go backend.
/// The Go API:
///   POST /openvas/targets
///   body: { "name": "...", "hosts": "...", "port_range": "..."? }
/// returns:
///   { "id": "<target-id>", "existed": true|false }
pub async fn create_target(
    name: &str,
    hosts: &str,
    port_range: Option<&str>,
) -> Result<Value> {
    let client = reqwest::Client::new();

    let mut body_map = Map::new();
    body_map.insert("name".into(), Value::String(name.to_string()));
    body_map.insert("hosts".into(), Value::String(hosts.to_string()));
    if let Some(pr) = port_range {
        if !pr.trim().is_empty() {
            body_map.insert("port_range".into(), Value::String(pr.to_string()));
        }
    }

    let resp = client
        .post("http://127.0.0.1:8080/openvas/targets")
        .json(&Value::Object(body_map))
        .send()
        .await?
        .error_for_status()?;

    let body: Value = resp.json().await?;
    Ok(body)
}

/// Create (or reuse) an OpenVAS task via the Go backend.
/// The Go API:
///   POST /openvas/tasks
///   body: { "name": "...", "config_id": "...", "target_id": "..." }
/// returns:
///   { "id": "<task-id>", "existed": true|false }
pub async fn create_task(
    name: &str,
    config_id: &str,
    target_id: &str,
) -> Result<Value> {
    let client = reqwest::Client::new();

    let mut body_map = Map::new();
    body_map.insert("name".into(), Value::String(name.to_string()));
    body_map.insert("config_id".into(), Value::String(config_id.to_string()));
    body_map.insert("target_id".into(), Value::String(target_id.to_string()));

    let resp = client
        .post("http://127.0.0.1:8080/openvas/tasks")
        .json(&Value::Object(body_map))
        .send()
        .await?
        .error_for_status()?;

    let body: Value = resp.json().await?;
    Ok(body)
}

/// Start an existing OpenVAS task via the Go backend.
/// The Go API:
///   POST /openvas/tasks/start
///   body: { "task_id": "..." }
/// returns:
///   { "task_id": "...", "response_raw": "<start_task_response XML>" }
pub async fn start_task(task_id: &str) -> Result<Value> {
    let client = reqwest::Client::new();

    let mut body_map = Map::new();
    body_map.insert("task_id".into(), Value::String(task_id.to_string()));

    let resp = client
        .post("http://127.0.0.1:8080/openvas/tasks/start")
        .json(&Value::Object(body_map))
        .send()
        .await?
        .error_for_status()?;

    let body: Value = resp.json().await?;
    Ok(body)
}

/// Get the current status/details for an existing OpenVAS task via the Go backend.
/// The Go API:
///   POST /openvas/tasks/status
///   body: { "task_id": "..." }
/// returns:
///   { "task_id": "...", "response_raw": "<get_tasks_response XML>" }
pub async fn get_task_status(task_id: &str) -> Result<Value> {
    let client = reqwest::Client::new();

    let mut body_map = Map::new();
    body_map.insert("task_id".into(), Value::String(task_id.to_string()));

    let resp = client
        .post("http://127.0.0.1:8080/openvas/tasks/status")
        .json(&Value::Object(body_map))
        .send()
        .await?
        .error_for_status()?;

    let body: Value = resp.json().await?;
    Ok(body)
}

/// Fetch the final OpenVAS report by report ID via the Go backend.
/// The Go API:
///   POST /openvas/reports
///   body: { "report_id": "..." }
/// returns:
///   { "report_id": "...", "response_raw": "<get_reports_response XML>" }
pub async fn get_report(report_id: &str) -> Result<Value> {
    let client = reqwest::Client::new();

    let mut body_map = Map::new();
    body_map.insert("report_id".into(), Value::String(report_id.to_string()));

    let resp = client
        .post("http://127.0.0.1:8080/openvas/reports")
        .json(&Value::Object(body_map))
        .send()
        .await?
        .error_for_status()?;

    let body: Value = resp.json().await?;
    Ok(body)
}

#[derive(Debug, Clone)]
pub enum OpenVASTaskProgressEvent {
    Ready(Value),
    Progress(Value),
    Done(Value),
}

/// Stream task progress from the Go backend (SSE).
///
/// GET /openvas/tasks/progress?task_id=...&interval_seconds=2
pub async fn task_progress_stream(
    task_id: &str,
    interval_seconds: Option<u64>,
) -> Result<mpsc::Receiver<OpenVASTaskProgressEvent>> {
    let client = reqwest::Client::new();

    let mut url = format!("http://127.0.0.1:8080/openvas/tasks/progress?task_id={}", task_id);
    if let Some(s) = interval_seconds {
        url.push_str(&format!("&interval_seconds={}", s));
    }

    let resp = client.get(url).send().await?.error_for_status()?;

    let (tx, rx) = mpsc::channel::<OpenVASTaskProgressEvent>(512);

    tokio::spawn(async move {
        let mut bytes = resp.bytes_stream();
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
                    if let Some(ev) = cur_event.take() {
                        let data = cur_data.join("\n");
                        cur_data.clear();

                        if let Ok(v) = serde_json::from_str::<Value>(&data) {
                            match ev.as_str() {
                                "ready" => {
                                    let _ = tx.send(OpenVASTaskProgressEvent::Ready(v)).await;
                                }
                                "progress" => {
                                    let _ = tx.send(OpenVASTaskProgressEvent::Progress(v)).await;
                                }
                                "done" => {
                                    let _ = tx.send(OpenVASTaskProgressEvent::Done(v)).await;
                                    return;
                                }
                                _ => {}
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
            }
        }
    });

    Ok(rx)
}

