use anyhow::Result;
use serde_json::Value;

use crate::services::openvas_wait_for_task;
use crate::api::openvas::OpenVASTaskProgressEvent;
use crate::Tool;
use crate::ExecutionContext;

/// Tool that waits for an OpenVAS task to complete, emitting progress updates.
pub struct OpenVASWaitForTaskTool;

#[async_trait::async_trait]
impl Tool for OpenVASWaitForTaskTool {
    fn name(&self) -> &'static str {
        "openvas_wait_for_task"
    }

    fn description(&self) -> &'static str {
        "Waits for an OpenVAS/GVM task to finish, streaming progress updates from the Go backend."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "task_id": {
                    "type": "string",
                    "description": "OpenVAS task ID to wait for."
                },
                "interval_seconds": {
                    "type": "integer",
                    "description": "Polling interval used by the Go backend SSE stream (1-60). Default: 2"
                }
            },
            "required": ["task_id"],
            "additionalProperties": false
        })
    }

    async fn execute(&self, input: Value, ctx: ExecutionContext) -> Result<Value> {
        let task_id = input
            .get("task_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing required field `task_id`"))?;

        let interval_seconds = input
            .get("interval_seconds")
            .and_then(|v| v.as_u64());

        // If client requested progress, stream ourselves so we can forward progress notifications.
        if ctx.progress_token().is_some() {
            let mut rx = crate::api::openvas::task_progress_stream(task_id, interval_seconds).await?;
            let mut sent_progress = 0.0f64;
            let mut tick = 0.0f64;

            while let Some(ev) = rx.recv().await {
                match ev {
                    OpenVASTaskProgressEvent::Ready(v) => {
                        let msg = v
                            .get("message")
                            .and_then(|m| m.as_str())
                            .unwrap_or("openvas progress stream ready")
                            .to_string();
                        // Initial progress note (keeps monotonic progress logic intact later).
                        ctx.notify_progress(0.0, None, Some(msg)).await;
                    }
                    OpenVASTaskProgressEvent::Progress(v) => {
                        // Try to extract percent from v.parsed.progress (0-100)
                        let parsed = v.get("parsed");
                        let percent = parsed
                            .and_then(|p| p.get("progress"))
                            .and_then(|p| p.as_f64());

                        let msg = parsed
                            .and_then(|p| p.get("message"))
                            .and_then(|m| m.as_str())
                            .map(|s| s.to_string());

                        if let Some(p) = percent {
                            // Enforce monotonic increase per MCP spec.
                            let mut next = p;
                            if next <= sent_progress {
                                next = (sent_progress + 0.01).min(100.0);
                            }
                            sent_progress = next;
                            ctx.notify_progress(sent_progress, Some(100.0), msg).await;
                        } else {
                            tick += 1.0;
                            sent_progress = tick;
                            ctx.notify_progress(sent_progress, None, msg).await;
                        }
                    }
                    OpenVASTaskProgressEvent::Done(v) => {
                        // Best-effort final tick.
                        if sent_progress < 100.0 {
                            ctx.notify_progress(100.0, Some(100.0), Some("openvas completed".to_string())).await;
                        }
                        return Ok(v);
                    }
                }
            }

            Ok(serde_json::json!({
                "task_id": task_id,
                "warning": "progress stream ended unexpectedly"
            }))
        } else {
            // Non-streaming: just wait and return final done payload.
            openvas_wait_for_task::openvas_wait_for_task(task_id, interval_seconds).await
        }
    }
}

