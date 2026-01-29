use anyhow::Result;
use serde_json::Value;

use crate::api::nmap;

/// Business-logic layer for a "normal" Nmap scan using the Go backend.
pub async fn nmap_normal_scan(target: &str, timing: Option<&str>) -> Result<Value> {
    // In the future we can add validation, logging, or result shaping here.
    nmap::scan_open_ports(target, timing).await
}

