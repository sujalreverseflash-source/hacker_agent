use anyhow::Result;
use serde_json::Value;

use crate::services::advanced_nmap_scan;
use crate::Tool;

/// Advanced Nmap tool with comprehensive options
pub struct AdvancedNmapTool;

#[async_trait::async_trait]
impl Tool for AdvancedNmapTool {
    fn name(&self) -> &'static str {
        "advanced_nmap_scan"
    }

    fn description(&self) -> &'static str {
        "Comprehensive Nmap scan with multiple options: timing, scan types, service detection, OS detection, scripts, and output formats."
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
                },
                "scan_type": {
                    "type": "string",
                    "description": "Type of scan to perform",
                    "enum": ["ping", "tcp_syn", "tcp_connect", "udp", "tcp_ack", "tcp_fin", "tcp_null", "tcp_xmas"]
                },
                "ports": {
                    "type": "string",
                    "description": "Port specification: '80,443', '1-1000', 'U:53,T:80-443', or 'all' for all ports"
                },
                "service_detection": {
                    "type": "boolean",
                    "description": "Enable service/version detection (-sV)"
                },
                "os_detection": {
                    "type": "boolean",
                    "description": "Enable OS detection (-O)"
                },
                "scripts": {
                    "type": "string",
                    "description": "Script names or categories: 'vuln', 'default', 'auth,discovery', or specific script names"
                },
                "output_format": {
                    "type": "string",
                    "description": "Output format for results",
                    "enum": ["normal", "xml", "json", "greppable", "all"]
                },
                "aggressive": {
                    "type": "boolean",
                    "description": "Enable aggressive scan options (-A): service detection, OS detection, scripts, and traceroute"
                },
                "traceroute": {
                    "type": "boolean", 
                    "description": "Enable traceroute (--traceroute)"
                },
                "flag_o": {
                    "type": "boolean",
                    "description": "Enable OS detection (-O)"
                },
                "flag_sc": {
                    "type": "boolean",
                    "description": "Enable default scripts (-sC)"
                },
                "flag_sv": {
                    "type": "boolean",
                    "description": "Enable service detection (-sV)"
                },
                "flag_traceroute": {
                    "type": "boolean",
                    "description": "Enable traceroute (--traceroute)"
                },
                "flag_a": {
                    "type": "boolean",
                    "description": "Enable aggressive scan (-A)"
                },
                "stealth_options": {
                    "type": "object",
                    "description": "Stealth and evasion options",
                    "properties": {
                        "decoys": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Decoy IPs (-D RND:10,ME,8.8.8.8)"
                        },
                        "source_port": {
                            "type": "integer",
                            "description": "Source port for packets (--source-port 53)"
                        },
                        "interface": {
                            "type": "string",
                            "description": "Network interface to use (-e eth0)"
                        },
                        "ttl": {
                            "type": "integer",
                            "description": "Time to live for packets (--ttl 64)"
                        },
                        "randomize_hosts": {
                            "type": "boolean",
                            "description": "Randomize target host order (--randomize-hosts)"
                        },
                        "spoof_ip": {
                            "type": "string",
                            "description": "Spoof source IP address (-S 192.168.1.1)"
                        },
                        "spoof_mac": {
                            "type": "string",
                            "description": "Spoof MAC address (--spoof-mac 0)"
                        }
                    }
                }
            },
            "required": ["target"],
            "additionalProperties": false
        })
    }

    async fn execute(&self, input: Value) -> Result<Value> {
        let target = input
            .get("target")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing required field `target`"))?;

        let timing = input.get("timing").and_then(|v| v.as_str());
        let scan_type = input.get("scan_type").and_then(|v| v.as_str());
        let ports = input.get("ports").and_then(|v| v.as_str());
        let service_detection = input.get("service_detection").and_then(|v| v.as_bool()).unwrap_or(false);
        let os_detection = input.get("os_detection").and_then(|v| v.as_bool()).unwrap_or(false);
        let scripts = input.get("scripts").and_then(|v| v.as_str());
        let output_format = input.get("output_format").and_then(|v| v.as_str());
        let aggressive = input.get("aggressive").and_then(|v| v.as_bool()).unwrap_or(false);
        let traceroute = input.get("traceroute").and_then(|v| v.as_bool()).unwrap_or(false);
        let flag_o = input.get("flag_o").and_then(|v| v.as_bool()).unwrap_or(false);
        let flag_sc = input.get("flag_sc").and_then(|v| v.as_bool()).unwrap_or(false);
        let flag_sv = input.get("flag_sv").and_then(|v| v.as_bool()).unwrap_or(false);
        let flag_traceroute = input.get("flag_traceroute").and_then(|v| v.as_bool()).unwrap_or(false);
        let flag_a = input.get("flag_a").and_then(|v| v.as_bool()).unwrap_or(false);
        let stealth_options = input.get("stealth_options");

        advanced_nmap_scan::advanced_nmap_scan(
            target,
            timing,
            scan_type,
            ports,
            service_detection,
            os_detection,
            scripts,
            output_format,
            aggressive,
            traceroute,
            flag_o,
            flag_sc,
            flag_sv,
            flag_traceroute,
            flag_a,
            stealth_options,
        ).await
    }
}

/// Quick scan tool for common use cases
pub struct QuickScanTool;

#[async_trait::async_trait]
impl Tool for QuickScanTool {
    fn name(&self) -> &'static str {
        "quick_scan"
    }

    fn description(&self) -> &'static str {
        "Fast network reconnaissance with common scan patterns (ping sweep, port scan, service detection)."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "target": {
                    "type": "string",
                    "description": "Target hostname, IP, or CIDR range."
                },
                "scan_type": {
                    "type": "string",
                    "description": "Quick scan type",
                    "enum": ["ping_sweep", "common_ports", "service_detection", "vuln_scan"],
                    "default": "common_ports"
                },
                "timing": {
                    "type": "string",
                    "description": "Speed: T3 (Normal) or T4 (Aggressive)",
                    "enum": ["T3", "T4"],
                    "default": "T4"
                }
            },
            "required": ["target"],
            "additionalProperties": false
        })
    }

    async fn execute(&self, input: Value) -> Result<Value> {
        let target = input
            .get("target")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing required field `target`"))?;

        let scan_type = input.get("scan_type").and_then(|v| v.as_str()).unwrap_or("common_ports");
        let timing = input.get("timing").and_then(|v| v.as_str()).unwrap_or("T4");

        advanced_nmap_scan::quick_scan(target, scan_type, timing).await
    }
}

/// Stealth scan tool for evasion techniques
pub struct StealthScanTool;

#[async_trait::async_trait]
impl Tool for StealthScanTool {
    fn name(&self) -> &'static str {
        "stealth_scan"
    }

    fn description(&self) -> &'static str {
        "Stealthy scans with evasion techniques (slow timing, decoys, fragmentation)."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "target": {
                    "type": "string",
                    "description": "Target hostname or IP address."
                },
                "stealth_level": {
                    "type": "string",
                    "description": "Stealth level",
                    "enum": ["low", "medium", "high", "maximum"],
                    "default": "medium"
                },
                "scan_type": {
                    "type": "string",
                    "description": "Stealth scan type",
                    "enum": ["tcp_fin", "tcp_null", "tcp_xmas", "tcp_ack", "tcp_syn"],
                    "default": "tcp_syn"
                },
                "use_decoys": {
                    "type": "boolean",
                    "description": "Use decoy hosts",
                    "default": true
                },
                "fragment_packets": {
                    "type": "boolean",
                    "description": "Fragment packets to evade IDS",
                    "default": false
                }
            },
            "required": ["target"],
            "additionalProperties": false
        })
    }

    async fn execute(&self, input: Value) -> Result<Value> {
        let target = input
            .get("target")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing required field `target`"))?;

        let stealth_level = input.get("stealth_level").and_then(|v| v.as_str()).unwrap_or("medium");
        let scan_type = input.get("scan_type").and_then(|v| v.as_str()).unwrap_or("tcp_syn");
        let use_decoys = input.get("use_decoys").and_then(|v| v.as_bool()).unwrap_or(true);
        let fragment_packets = input.get("fragment_packets").and_then(|v| v.as_bool()).unwrap_or(false);

        advanced_nmap_scan::stealth_scan(target, stealth_level, scan_type, use_decoys, fragment_packets).await
    }
}

/// Comprehensive scan tool - full port scan with service/OS detection
pub struct ComprehensiveScanTool;

#[async_trait::async_trait]
impl Tool for ComprehensiveScanTool {
    fn name(&self) -> &'static str {
        "comprehensive_scan"
    }

    fn description(&self) -> &'static str {
        "Full comprehensive scan: all 65535 ports with service detection, OS detection, and scripts. Use for thorough security assessment."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "target": {
                    "type": "string",
                    "description": "Target hostname or IP address to scan."
                },
                "include_vuln": {
                    "type": "boolean",
                    "description": "Include vulnerability scripts (vuln category). Default: false",
                    "default": false
                }
            },
            "required": ["target"],
            "additionalProperties": false
        })
    }

    async fn execute(&self, input: Value) -> Result<Value> {
        let target = input
            .get("target")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing required field `target`"))?;

        let include_vuln = input.get("include_vuln").and_then(|v| v.as_bool()).unwrap_or(false);

        advanced_nmap_scan::comprehensive_scan(target, include_vuln).await
    }
}

/// Network discovery tool - subnet enumeration
pub struct NetworkDiscoveryTool;

#[async_trait::async_trait]
impl Tool for NetworkDiscoveryTool {
    fn name(&self) -> &'static str {
        "network_discovery"
    }

    fn description(&self) -> &'static str {
        "Network discovery scan for subnet enumeration. Finds live hosts and checks common ports (22, 80, 443, 3389, 8080)."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "subnet": {
                    "type": "string",
                    "description": "Target subnet in CIDR notation (e.g., '192.168.1.0/24') or IP range."
                },
                "timing": {
                    "type": "string",
                    "description": "Timing template: T3 (Normal) or T4 (Aggressive). Default: T4",
                    "enum": ["T3", "T4"],
                    "default": "T4"
                }
            },
            "required": ["subnet"],
            "additionalProperties": false
        })
    }

    async fn execute(&self, input: Value) -> Result<Value> {
        let subnet = input
            .get("subnet")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing required field `subnet`"))?;

        let timing = input.get("timing").and_then(|v| v.as_str()).unwrap_or("T4");

        advanced_nmap_scan::network_discovery(subnet, timing).await
    }
}
