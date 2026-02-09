use anyhow::Result;
use serde_json::Value;

use crate::api::openvas;
use crate::api::openvas::OpenVASTaskProgressEvent;

/// Wait for an OpenVAS task to finish, streaming progress events from the Go backend.
///
/// Returns the final "done" payload (includes parsed fields if the Go backend provides them).
pub async fn openvas_wait_for_task(
    task_id: &str,
    interval_seconds: Option<u64>,
) -> Result<Value> {
    let mut rx = openvas::task_progress_stream(task_id, interval_seconds).await?;

    let mut last_progress: Option<Value> = None;
    while let Some(ev) = rx.recv().await {
        match ev {
            OpenVASTaskProgressEvent::Ready(_) => {}
            OpenVASTaskProgressEvent::Progress(v) => {
                last_progress = Some(v);
            }
            OpenVASTaskProgressEvent::Done(v) => {
                return Ok(v);
            }
        }
    }

    Ok(serde_json::json!({
        "task_id": task_id,
        "warning": "progress stream ended unexpectedly",
        "last_progress": last_progress
    }))
}

