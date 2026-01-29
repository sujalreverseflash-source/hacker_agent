package main

import (
	"encoding/json"
	"log"
	"net/http"
	"os/exec"
	"strings"

	"github.com/joho/godotenv"
)

type scanRequest struct {
	Target           string `json:"target"`
	Timing           string `json:"timing,omitempty"`
	ScanType         string `json:"scan_type,omitempty"`
	Ports            string `json:"ports,omitempty"`
	ServiceDetection bool   `json:"service_detection,omitempty"`
	OSDetection      bool   `json:"os_detection,omitempty"`
	Scripts          string `json:"scripts,omitempty"`
	OutputFormat     string `json:"output_format,omitempty"`
	Aggressive       bool   `json:"aggressive,omitempty"` // -A flag
	Traceroute       bool   `json:"traceroute,omitempty"` // --traceroute
	// Direct Nmap flags
	FlagO          bool                   `json:"flag_o,omitempty"`          // -O (OS detection)
	FlagSC         bool                   `json:"flag_sc,omitempty"`         // -sC (default scripts)
	FlagSV         bool                   `json:"flag_sv,omitempty"`         // -sV (service detection)
	FlagTraceroute bool                   `json:"flag_traceroute,omitempty"` // --traceroute
	FlagA          bool                   `json:"flag_a,omitempty"`          // -A (aggressive)
	StealthOptions map[string]interface{} `json:"stealth_options,omitempty"`
}

type scanResponse struct {
	Target    string `json:"target"`
	RawOutput string `json:"raw_output"`
}

func scanOpenPortsHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
		return
	}

	var req scanRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "invalid JSON body", http.StatusBadRequest)
		return
	}
	req.Target = strings.TrimSpace(req.Target)
	if req.Target == "" {
		http.Error(w, "target is required", http.StatusBadRequest)
		return
	}

	// Build nmap command with all options
	var cmdArgs []string

	// Add timing template
	timingTemplate := req.Timing
	if timingTemplate == "" {
		timingTemplate = "T2"
	}
	validTimings := map[string]bool{"T0": true, "T1": true, "T2": true, "T3": true, "T4": true, "T5": true}
	if !validTimings[timingTemplate] {
		http.Error(w, "invalid timing template. Must be one of: T0, T1, T2, T3, T4, T5", http.StatusBadRequest)
		return
	}
	cmdArgs = append(cmdArgs, "-"+timingTemplate)

	// Add scan type
	if req.ScanType != "" {
		validScanTypes := map[string]string{
			"ping":        "-sn",
			"tcp_syn":     "-sS",
			"tcp_connect": "-sT",
			"udp":         "-sU",
			"tcp_ack":     "-sA",
			"tcp_fin":     "-sF",
			"tcp_null":    "-sN",
			"tcp_xmas":    "-sX",
		}
		if scanType, exists := validScanTypes[req.ScanType]; exists {
			cmdArgs = append(cmdArgs, scanType)
		}
	}

	// Add port specification
	if req.Ports != "" {
		cmdArgs = append(cmdArgs, "-p", req.Ports)
	}

	// Add service detection
	if req.ServiceDetection {
		cmdArgs = append(cmdArgs, "-sV")
	}

	// Add OS detection
	if req.OSDetection {
		cmdArgs = append(cmdArgs, "-O")
	}

	// Add script scanning
	if req.Scripts != "" {
		cmdArgs = append(cmdArgs, "--script", req.Scripts)
	}

	// Add output format
	if req.OutputFormat != "" {
		validFormats := map[string]string{
			"xml":       "-oX",
			"json":      "-oJ",
			"greppable": "-oG",
			"all":       "-oA",
		}
		if format, exists := validFormats[req.OutputFormat]; exists {
			cmdArgs = append(cmdArgs, format)
		}
	}

	// Add direct Nmap flags
	if req.FlagO {
		cmdArgs = append(cmdArgs, "-O")
	}
	if req.FlagSC {
		cmdArgs = append(cmdArgs, "-sC")
	}
	if req.FlagSV {
		cmdArgs = append(cmdArgs, "-sV")
	}
	if req.FlagTraceroute {
		cmdArgs = append(cmdArgs, "--traceroute")
	}
	if req.FlagA {
		cmdArgs = append(cmdArgs, "-A")
	}

	// Add aggressive scan options (-A flag) - for backward compatibility
	if req.Aggressive && !req.FlagA {
		cmdArgs = append(cmdArgs, "-A")
	}

	// Add traceroute separately (in case user wants it without -A)
	if req.Traceroute && !req.FlagTraceroute && !req.Aggressive && !req.FlagA {
		cmdArgs = append(cmdArgs, "--traceroute")
	}

	// Add target
	cmdArgs = append(cmdArgs, req.Target)

	// Build command
	cmd := exec.Command("nmap", cmdArgs...)
	out, err := cmd.CombinedOutput()
	if err != nil {
		// Still return whatever output we got, plus the error text.
		log.Printf("nmap error for target %s: %v", req.Target, err)
	}

	resp := scanResponse{
		Target:    req.Target,
		RawOutput: string(out),
	}

	w.Header().Set("Content-Type", "application/json")
	if err := json.NewEncoder(w).Encode(resp); err != nil {
		log.Printf("failed to encode response: %v", err)
	}
}

func main() {
	// Load environment variables from .env so OpenVAS auth/config
	// is available without manually exporting each time.
	_ = godotenv.Load(".env")

	mux := http.NewServeMux()
	mux.HandleFunc("/scan-open-ports", scanOpenPortsHandler)

	// Modular OpenVAS APIs.
	openVASService := NewOpenVASServiceFromEnv()
	mux.Handle("/openvas/version", openVASVersionHandler(openVASService))
	mux.Handle("/openvas/configs", openVASConfigsHandler(openVASService))

	addr := ":8081"
	log.Printf("Go backend listening on %s", addr)
	if err := http.ListenAndServe(addr, mux); err != nil {
		log.Fatalf("server failed: %v", err)
	}
}
