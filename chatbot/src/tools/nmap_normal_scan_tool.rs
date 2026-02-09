use anyhow::Result;
use serde_json::Value;

use crate::services::nmap_normal_scan;
use crate::api::nmap::NmapStreamEvent;
use crate::Tool;
use crate::ExecutionContext;

/// Tool that exposes a "normal" Nmap open-port scan via the Go backend.
pub struct NmapOpenPortsTool;

#[async_trait::async_trait]
impl Tool for NmapOpenPortsTool {
    fn name(&self) -> &'static str {
        "nmap_open_ports"
    }

    fn description(&self) -> &'static str {
        "Scans open TCP ports on a given target with optional timing template (T0-T5)."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "target": {
                    "type": "string",
                    "description": "Target hostname or IP address to scan."
                },
                "timing": {
                    "type": "string",
                    "description": "Nmap timing template: T0 (Paranoid), T1 (Sneaky), T2 (Polite), T3 (Normal), T4 (Aggressive), T5 (Insane). Default: T2",
                    "enum": ["T0", "T1", "T2", "T3", "T4", "T5"]
                }
            },
            "required": ["target"],
            "additionalProperties": false
        })
    }

    async fn execute(&self, input: Value, ctx: ExecutionContext) -> Result<Value> {
        let target = input
            .get("target")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing required field `target`"))?;
            
        let timing = input
            .get("timing")
            .and_then(|v| v.as_str());

        if ctx.progress_token().is_some() {
            let mut rx = nmap_normal_scan::nmap_normal_scan_stream(target, timing).await?;
            let mut progress = 0.0f64;
            let mut lines: Vec<String> = Vec::new();

            while let Some(ev) = rx.recv().await {
                match ev {
                    NmapStreamEvent::Ready(ready) => {
                        // Read fields so they aren't "unused", but keep noise low.
                        let msg = ready
                            .get("message")
                            .and_then(|m| m.as_str())
                            .unwrap_or("nmap stream ready")
                            .to_string();
                        ctx.notify_progress(progress, None, Some(msg)).await;
                    }
                    NmapStreamEvent::Output(o) => {
                        progress += 1.0;
                        let line = format!("[{} {}] {}", o.timestamp, o.stream, o.line);
                        lines.push(line.clone());
                        ctx.notify_progress(progress, None, Some(line)).await;
                    }
                    NmapStreamEvent::Done(done) => {
                        // One final "completed" tick.
                        progress += 1.0;
                        ctx.notify_progress(progress, None, Some("nmap completed".to_string())).await;
                        return Ok(serde_json::json!({
                            "target": target,
                            "raw_output": lines.join("\n"),
                            "done": done
                        }));
                    }
                }
            }

            Ok(serde_json::json!({
                "target": target,
                "raw_output": lines.join("\n"),
                "warning": "stream ended before done event"
            }))
        } else {
            nmap_normal_scan::nmap_normal_scan(target, timing).await
        }
    }
}

