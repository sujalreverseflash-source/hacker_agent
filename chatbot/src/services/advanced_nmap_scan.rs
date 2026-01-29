use anyhow::Result;
use serde_json::{json, Value};

use crate::api::nmap;

/// Comprehensive Nmap scan with all options
pub async fn advanced_nmap_scan(
    target: &str,
    timing: Option<&str>,
    scan_type: Option<&str>,
    ports: Option<&str>,
    service_detection: bool,
    os_detection: bool,
    scripts: Option<&str>,
    output_format: Option<&str>,
    aggressive: bool,
    traceroute: bool,
    flag_o: bool,
    flag_sc: bool,
    flag_sv: bool,
    flag_traceroute: bool,
    flag_a: bool,
    stealth_options: Option<&Value>,
) -> Result<Value> {
    let mut body = json!({
        "target": target
    });

    // Add optional parameters
    if let Some(t) = timing {
        body["timing"] = json!(t);
    }
    if let Some(st) = scan_type {
        body["scan_type"] = json!(st);
    }
    if let Some(p) = ports {
        body["ports"] = json!(p);
    }
    if service_detection {
        body["service_detection"] = json!(true);
    }
    if os_detection {
        body["os_detection"] = json!(true);
    }
    if let Some(s) = scripts {
        body["scripts"] = json!(s);
    }
    if let Some(of) = output_format {
        body["output_format"] = json!(of);
    }
    if let Some(so) = stealth_options {
        body["stealth_options"] = so.clone();
    }
    if aggressive {
        body["aggressive"] = json!(true);
    }
    if traceroute {
        body["traceroute"] = json!(true);
    }
    if flag_o {
        body["flag_o"] = json!(true);
    }
    if flag_sc {
        body["flag_sc"] = json!(true);
    }
    if flag_sv {
        body["flag_sv"] = json!(true);
    }
    if flag_traceroute {
        body["flag_traceroute"] = json!(true);
    }
    if flag_a {
        body["flag_a"] = json!(true);
    }

    nmap::advanced_scan(&body).await
}

/// Quick scan presets for common scenarios
pub async fn quick_scan(target: &str, scan_type: &str, timing: &str) -> Result<Value> {
    let body = match scan_type {
        "ping_sweep" => json!({
            "target": target,
            "timing": timing,
            "scan_type": "ping"
        }),
        "common_ports" => json!({
            "target": target,
            "timing": timing,
            "scan_type": "tcp_syn",
            "ports": "1-1000",
            "service_detection": true
        }),
        "service_detection" => json!({
            "target": target,
            "timing": timing,
            "scan_type": "tcp_syn",
            "service_detection": true,
            "os_detection": true
        }),
        "vuln_scan" => json!({
            "target": target,
            "timing": timing,
            "scan_type": "tcp_syn",
            "service_detection": true,
            "scripts": "vuln"
        }),
        _ => json!({
            "target": target,
            "timing": timing
        })
    };

    nmap::advanced_scan(&body).await
}

/// Stealth scan with evasion techniques
pub async fn stealth_scan(
    target: &str,
    stealth_level: &str,
    scan_type: &str,
    use_decoys: bool,
    fragment_packets: bool,
) -> Result<Value> {
    let (timing, decoys, ttl) = match stealth_level {
        "low" => ("T3", None, None),
        "medium" => ("T2", 
            if use_decoys { Some(json!(["RND:5", "ME"])) } else { None },
            Some(64)
        ),
        "high" => ("T1",
            if use_decoys { Some(json!(["RND:10", "8.8.8.8", "ME"])) } else { None },
            Some(128)
        ),
        "maximum" => ("T0",
            if use_decoys { Some(json!(["RND:15", "8.8.8.8", "1.1.1.1", "ME"])) } else { None },
            Some(255)
        ),
        _ => ("T2", None, None)
    };

    let mut stealth_opts = json!({});
    if let Some(d) = decoys {
        stealth_opts["decoys"] = d;
    }
    if let Some(t) = ttl {
        stealth_opts["ttl"] = json!(t);
    }
    if fragment_packets {
        stealth_opts["fragment_packets"] = json!(true);
    }

    let body = json!({
        "target": target,
        "timing": timing,
        "scan_type": scan_type,
        "stealth_options": stealth_opts
    });

    nmap::advanced_scan(&body).await
}

/// Comprehensive scan with multiple techniques
pub async fn comprehensive_scan(target: &str, include_vuln: bool) -> Result<Value> {
    let body = json!({
        "target": target,
        "timing": "T3",
        "scan_type": "tcp_syn",
        "ports": "1-65535",
        "service_detection": true,
        "os_detection": true,
        "scripts": if include_vuln { "default,vuln" } else { "default" },
        "output_format": "xml"
    });

    nmap::advanced_scan(&body).await
}

/// Network discovery scan for subnet enumeration
pub async fn network_discovery(subnet: &str, timing: &str) -> Result<Value> {
    let body = json!({
        "target": subnet,
        "timing": timing,
        "scan_type": "ping",
        "ports": "22,80,443,3389,8080"
    });

    nmap::advanced_scan(&body).await
}
