use anyhow::Result;
use serde_json::Value;
use tokio::sync::mpsc;

use crate::api::nmap;

/// Business-logic layer for a "normal" Nmap scan using the Go backend.
pub async fn nmap_normal_scan(target: &str, timing: Option<&str>) -> Result<Value> {
    // In the future we can add validation, logging, or result shaping here.
    nmap::scan_open_ports(target, timing).await
}

/// Streaming version of the normal scan that yields incremental output lines.
pub async fn nmap_normal_scan_stream(
    target: &str,
    timing: Option<&str>,
) -> Result<mpsc::Receiver<nmap::NmapStreamEvent>> {
    nmap::scan_open_ports_stream(target, timing).await
}

