use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NmapScanRequest {
    pub target: String,
    pub timing: Option<TimingTemplate>,
    pub scan_type: Option<ScanType>,
    pub port_specification: Option<PortSpec>,
    pub service_detection: Option<ServiceDetection>,
    pub os_detection: bool,
    pub script_scan: Option<ScriptScan>,
    pub output_format: Option<OutputFormat>,
    pub stealth_options: Option<StealthOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimingTemplate {
    T0, // Paranoid
    T1, // Sneaky
    T2, // Polite
    T3, // Normal
    T4, // Aggressive
    T5, // Insane
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScanType {
    PingScan,              // -sn
    TcpScan,               // -sS (SYN)
    TcpConnectScan,        // -sT
    UdpScan,               // -sU
    TcpSynScan,            // -sS
    TcpAckScan,            // -sA
    TcpWindowScan,         // -sW
    TcpMaimonScan,         // -sM
    TcpFinScan,            // -sF
    TcpNullScan,           // -sN
    TcpXmasScan,           // -sX
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortSpec {
    All,                   // -p-
    TopPorts(u16),         // --top-ports N
    Specific(Vec<String>), // -p 80,443,8080
    Range(String),         // -p 1-1000
    Common,                // default
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDetection {
    pub enabled: bool,     // -sV
    pub intensity: Option<u8>, // --version-intensity 0-9
    pub all_ports: bool,    // --all-ports
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptScan {
    pub scripts: Vec<String>, // --script
    pub args: Option<String>,   // --script-args
    pub categories: Vec<ScriptCategory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScriptCategory {
    Auth, Default, Discovery, Dos, Exploit, External, Fuzzer, Intrusive,
    Malware, Safe, Version, Vuln, Brute, Broadcast, Meterpreter, Sniffer
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    Normal,    // default
    Xml,       // -oX
    Json,      // -oJ
    Greppable, // -oG
    All,       // -oA
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StealthOptions {
    pub decoys: Option<Vec<String>>,    // -D
    pub source_port: Option<u16>,       // --source-port
    pub interface: Option<String>,      // -e
    pub ttl: Option<u8>,               // --ttl
    pub randomize_hosts: bool,         // --randomize-hosts
    pub spoof_ip: Option<String>,       // -S
    pub spoof_mac: Option<String>,      // --spoof-mac
}

impl NmapScanRequest {
    pub fn build_command(&self) -> Vec<String> {
        let mut cmd = vec!["nmap".to_string()];
        
        // Add timing template
        if let Some(timing) = &self.timing {
            cmd.push(format!("-T{}", match timing {
                TimingTemplate::T0 => "0",
                TimingTemplate::T1 => "1", 
                TimingTemplate::T2 => "2",
                TimingTemplate::T3 => "3",
                TimingTemplate::T4 => "4",
                TimingTemplate::T5 => "5",
            }));
        }
        
        // Add scan type
        if let Some(scan_type) = &self.scan_type {
            cmd.push(match scan_type {
                ScanType::PingScan => "-sn".to_string(),
                ScanType::TcpScan => "-sS".to_string(),
                ScanType::TcpConnectScan => "-sT".to_string(),
                ScanType::UdpScan => "-sU".to_string(),
                ScanType::TcpSynScan => "-sS".to_string(),
                ScanType::TcpAckScan => "-sA".to_string(),
                ScanType::TcpWindowScan => "-sW".to_string(),
                ScanType::TcpMaimonScan => "-sM".to_string(),
                ScanType::TcpFinScan => "-sF".to_string(),
                ScanType::TcpNullScan => "-sN".to_string(),
                ScanType::TcpXmasScan => "-sX".to_string(),
            });
        }
        
        // Add port specification
        if let Some(ports) = &self.port_specification {
            match ports {
                PortSpec::All => cmd.push("-p-".to_string()),
                PortSpec::TopPorts(n) => cmd.push(format!("--top-ports {}", n)),
                PortSpec::Specific(port_list) => cmd.push(format!("-p {}", port_list.join(","))),
                PortSpec::Range(range) => cmd.push(format!("-p {}", range)),
                PortSpec::Common => {}, // default, no flag needed
            }
        }
        
        // Add service detection
        if let Some(service_detection) = &self.service_detection {
            if service_detection.enabled {
                cmd.push("-sV".to_string());
                if let Some(intensity) = service_detection.intensity {
                    cmd.push(format!("--version-intensity {}", intensity));
                }
                if service_detection.all_ports {
                    cmd.push("--all-ports".to_string());
                }
            }
        }
        
        // Add OS detection
        if self.os_detection {
            cmd.push("-O".to_string());
        }
        
        // Add script scan
        if let Some(script_scan) = &self.script_scan {
            if !script_scan.scripts.is_empty() {
                cmd.push(format!("--script {}", script_scan.scripts.join(",")));
            }
            if !script_scan.categories.is_empty() {
                let categories: Vec<String> = script_scan.categories.iter()
                    .map(|c| format!("{:?}", c).to_lowercase())
                    .collect();
                cmd.push(format!("--script {}", categories.join(",")));
            }
            if let Some(args) = &script_scan.args {
                cmd.push(format!("--script-args {}", args));
            }
        }
        
        // Add output format
        if let Some(output_format) = &self.output_format {
            match output_format {
                OutputFormat::Xml => cmd.push("-oX".to_string()),
                OutputFormat::Json => cmd.push("-oJ".to_string()),
                OutputFormat::Greppable => cmd.push("-oG".to_string()),
                OutputFormat::All => cmd.push("-oA".to_string()),
                OutputFormat::Normal => {}, // default
            }
        }
        
        // Add stealth options
        if let Some(stealth) = &self.stealth_options {
            if let Some(decoys) = &stealth.decoys {
                cmd.push(format!("-D {}", decoys.join(",")));
            }
            if let Some(source_port) = stealth.source_port {
                cmd.push(format!("--source-port {}", source_port));
            }
            if let Some(interface) = &stealth.interface {
                cmd.push(format!("-e {}", interface));
            }
            if let Some(ttl) = stealth.ttl {
                cmd.push(format!("--ttl {}", ttl));
            }
            if stealth.randomize_hosts {
                cmd.push("--randomize-hosts".to_string());
            }
            if let Some(spoof_ip) = &stealth.spoof_ip {
                cmd.push(format!("-S {}", spoof_ip));
            }
            if let Some(spoof_mac) = &stealth.spoof_mac {
                cmd.push(format!("--spoof-mac {}", spoof_mac));
            }
        }
        
        // Add target
        cmd.push(self.target.clone());
        
        cmd
    }
}
