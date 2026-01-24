# Cyber Chatbot – Nmap MCP Tools (Design Doc)

This project is intended to be a **Rust-based MCP-style service** that lets an AI safely run common **Nmap** scans as “tools”.

The goal is:

- Expose a **small, well-documented set of Nmap scan patterns** as tools.
- Each tool has **clear inputs, behavior, and risks**.
- The MCP server (in Rust) accepts **JSON-RPC requests** (e.g. `tools/list`, `tools/call`) and internally runs the appropriate `nmap` command.

> **Important**: You must only run these tools on systems and networks you are authorized to test.

---

## Requirements

- `nmap` installed and accessible on the system PATH.
- A Rust binary (MCP server) that:
  - Listens on stdin/stdout for JSON messages.
  - Validates parameters.
  - Executes `nmap` with safe, pre-defined argument patterns.
  - Returns the scan output (or parsed results) as JSON.

This README documents **what tools exist and how they map to `nmap`**, so you or an AI agent can reason about them.

---

## Tool Catalog (Planned)

Each tool represents a **fixed Nmap scan pattern** with parameters.

| Tool name              | Nmap base command pattern                               | Purpose                                           |
|------------------------|---------------------------------------------------------|---------------------------------------------------|
| `ping_sweep`           | `nmap -sn <target_range>`                               | Discover live hosts in a range/subnet            |
| `quick_tcp_scan`       | `nmap -T4 <target>`                                     | Fast scan of common TCP ports                    |
| `full_tcp_scan`        | `nmap -p- -T4 <target>`                                 | Full TCP port scan (1–65535)                     |
| `service_version_scan` | `nmap -sV <target> [-p <ports>]`                        | Detect services and versions on open TCP ports   |
| `vuln_scripts_scan`    | `nmap --script vuln -sV <target>`                      | Run Nmap vuln-related NSE scripts                |
| `os_detection_scan`    | `sudo nmap -O <target>`                                 | Attempt to fingerprint the target OS             |
| `basic_udp_scan`       | `sudo nmap -sU --top-ports <N> <target>`               | Check top UDP ports for common services          |

Below are detailed descriptions, use cases, and examples for each.

---

## 1. `ping_sweep`

**Base Nmap command**

```bash
nmap -sn <target_range>
```

**Inputs**

- `target_range` (string): IP range or subnet, for example:
  - `192.168.1.0/24`
  - `10.0.0.1-50`

**What it does**

- Sends host discovery probes (ICMP echo, ARP on local net, etc.).
- Does **not** perform port scanning; only tells you which hosts are **up**.

**Typical use case**

- Before deeper scanning, the AI wants an inventory of which hosts exist in a given subnet.
- Example: “List all live hosts in `192.168.1.0/24` so we can decide what to scan next.”

**Manual CLI example**

```bash
nmap -sn 192.168.1.0/24
```

Sample output snippet:

```text
Nmap scan report for 192.168.1.10
Host is up (0.0030s latency).
Nmap scan report for 192.168.1.20
Host is up (0.0025s latency).
```

In an MCP tool, this raw output (or a parsed list of IPs) would be returned as JSON.

---

## 2. `quick_tcp_scan`

**Base Nmap command**

```bash
nmap -T4 <target>
```

**Inputs**

- `target` (string): Single IP, hostname, or a small range.

**What it does**

- Scans the **most common TCP ports** on the target.
- Uses timing profile `-T4` for reasonably fast scanning on typical networks.

**Typical use case**

- Quick check to see **what services are exposed** on a host.
- Example: “What ports are open on `10.0.0.5` so I know if it’s running SSH/HTTP/etc.?”

**Manual CLI example**

```bash
nmap -T4 10.0.0.5
```

Sample output snippet:

```text
PORT     STATE  SERVICE
22/tcp   open   ssh
80/tcp   open   http
443/tcp  open   https
```

In an MCP context, this could be parsed into a JSON array of `{port, protocol, state, service}`.

---

## 3. `full_tcp_scan`

**Base Nmap command**

```bash
nmap -p- -T4 <target>
```

**Inputs**

- `target` (string): Single host (recommended) or very small range.

**What it does**

- Scans **all 65535 TCP ports**.
- More thorough than `quick_tcp_scan` but slower.

**Typical use case**

- When the AI needs to ensure it hasn’t missed any **non-standard ports**.
- Example: “Enumerate all open TCP ports on this CTF target `10.10.10.10`.”

**Manual CLI example**

```bash
nmap -p- -T4 10.10.10.10
```

Sample output snippet:

```text
PORT      STATE SERVICE
21/tcp    open  ftp
22/tcp    open  ssh
80/tcp    open  http
8080/tcp  open  http-proxy
```

In a tool response, the AI would receive a full list of open ports and can then decide which to investigate further.

---

## 4. `service_version_scan`

**Base Nmap command**

```bash
nmap -sV <target>
# or, with ports:
nmap -sV -p <ports> <target>
```

**Inputs**

- `target` (string): Host or IP.
- `ports` (optional string): Port spec, e.g. `"22,80,443,8080-8090"`.

**What it does**

- Performs **service and version detection** (e.g. “OpenSSH 8.9p1”, “Apache httpd 2.4.58”).
- Helps the AI map open ports to specific software.

**Typical use case**

- After discovering open ports, the AI uses this tool to learn **what is actually running** there.
- Example: “Identify versions of services on `10.0.0.5` so I can look for vulnerabilities.”

**Manual CLI examples**

```bash
# Scan all common ports with version detection
nmap -sV 10.0.0.5

# Scan specific ports with version detection
nmap -sV -p 22,80,443 10.0.0.5
```

Sample output snippet:

```text
PORT   STATE SERVICE VERSION
22/tcp open  ssh     OpenSSH 8.9p1 Ubuntu 3ubuntu0.3
80/tcp open  http    Apache httpd 2.4.58 ((Ubuntu))
```

The MCP tool can convert this into structured JSON (e.g. `{port, service, version, product, extra_info}`).

---

## 5. `vuln_scripts_scan`

**Base Nmap command**

```bash
nmap --script vuln -sV <target>
```

**Inputs**

- `target` (string): Host or IP.

**What it does**

- Runs Nmap’s **vulnerability-related NSE scripts** on detected services.
- Uses `-sV` to get the necessary service information for scripts to work.

**Typical use case**

- AI wants a **quick vulnerability check** on a host after identifying its services.
- Example: “Check `10.0.0.5` for known issues using default vuln scripts.”

**Manual CLI example**

```bash
nmap --script vuln -sV 10.0.0.5
```

Sample output snippet (simplified):

```text
PORT   STATE SERVICE VERSION
80/tcp open  http    Apache httpd 2.4.58
| http-vuln-cve2017-5638:
|   VULNERABLE:
|   Apache Struts2 S2-045 remote command execution
|_  References: https://cvedetails.com/cve/CVE-2017-5638/
```

The MCP tool should return both **raw output** and (ideally) a **parsed list of findings** so the AI can reason about them.

---

## 6. `os_detection_scan`

**Base Nmap command**

```bash
sudo nmap -O <target>
```

**Inputs**

- `target` (string): Host or IP.

**What it does**

- Tries to **fingerprint the operating system** of the target host.
- Requires elevated privileges on most systems.

**Typical use case**

- AI wants to know whether a host is running **Windows, Linux, BSD, etc.**, and possibly the version family.
- Example: “Determine OS for `192.168.1.10` to plan follow-up checks.”

**Manual CLI example**

```bash
sudo nmap -O 192.168.1.10
```

Sample output snippet:

```text
OS details: Linux 5.4 - 5.15
Network Distance: 1 hop
```

The tool can expose OS guesses as fields like `os_family`, `os_details`, `accuracy`.

---

## 7. `basic_udp_scan`

**Base Nmap command**

```bash
sudo nmap -sU --top-ports <N> <target>
```

**Inputs**

- `target` (string): Host or IP.
- `N` (integer): Number of top UDP ports to scan (e.g. `50`, `100`).

**What it does**

- Scans the **top N most common UDP ports**.
- Useful to spot services like DNS (53/udp), NTP (123/udp), SNMP (161/udp), etc.

**Typical use case**

- AI wants a **lightweight UDP check** without doing a very slow full UDP scan.
- Example: “Check top 50 UDP ports on `10.0.0.5` for common services.”

**Manual CLI example**

```bash
sudo nmap -sU --top-ports 50 10.0.0.5
```

Sample output snippet:

```text
PORT    STATE         SERVICE
53/udp  open          domain
123/udp open          ntp
161/udp open|filtered snmp
```

The tool should represent each UDP result with port, state, and service.

---

## Example MCP-style JSON Calls (Conceptual)

The Rust MCP server is expected to implement something like:

- `tools/list` – list all available tools and descriptions.
- `tools/call` – call a specific tool with JSON parameters.

### List tools

**Request**

```json
{
  "id": 1,
  "method": "tools/list",
  "params": {}
}
```

**Response (example)**

```json
{
  "id": 1,
  "result": {
    "tools": [
      { "name": "ping_sweep", "description": "Discover live hosts in a subnet using nmap -sn." },
      { "name": "quick_tcp_scan", "description": "Fast common TCP port scan using nmap -T4." },
      { "name": "full_tcp_scan", "description": "Full 1–65535 TCP port scan using nmap -p- -T4." },
      { "name": "service_version_scan", "description": "Detect services and versions with nmap -sV." },
      { "name": "vuln_scripts_scan", "description": "Run Nmap vuln scripts with nmap --script vuln -sV." },
      { "name": "os_detection_scan", "description": "Attempt OS fingerprinting with nmap -O." },
      { "name": "basic_udp_scan", "description": "Top-N UDP ports scan with nmap -sU --top-ports." }
    ]
  }
}
```

### Call a specific tool (example: `quick_tcp_scan`)

**Request**

```json
{
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "quick_tcp_scan",
    "input": {
      "target": "10.0.0.5"
    }
  }
}
```

**Response (conceptual)**

```json
{
  "id": 2,
  "result": {
    "output": {
      "target": "10.0.0.5",
      "scanner": "nmap",
      "summary": "3 open ports",
      "ports": [
        { "port": 22, "protocol": "tcp", "state": "open", "service": "ssh" },
        { "port": 80, "protocol": "tcp", "state": "open", "service": "http" },
        { "port": 443, "protocol": "tcp", "state": "open", "service": "https" }
      ],
      "raw_output": "…full nmap text output here…"
    }
  }
}
```

This gives the AI a **structured view** of the results plus the **raw Nmap output** for reference.

---

## Next Steps for Implementation

1. Implement a Rust `Tool` trait and `ToolRegistry` that define:
   - `name()`, `description()`, and an async `execute(input: serde_json::Value)`.
2. Implement one Nmap-backed tool at a time (e.g. `ping_sweep`), validating inputs before building the command.
3. Parse Nmap’s output into structured JSON where practical, while also returning the raw text.
4. Add guardrails:
   - Limit target ranges.
   - Require confirmation or specific configuration for more aggressive scans.

Once these are implemented, this README serves as the **contract** describing what each tool does and how an AI (or a human) can use them. 
next...nessus.

Once these are implemented, this README serves as the **contract** describing what each tool does and how an AI (or a human) can use them. 

