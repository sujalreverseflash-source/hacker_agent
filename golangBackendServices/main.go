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

	cmdArgs, err := buildNmapArgs(req)
	if err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

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
	mux.HandleFunc("/scan-open-ports/stream", scanOpenPortsStreamHandler)
	mux.HandleFunc("/amass/enum", amassEnumHandler)
	mux.Handle("/amass/version", amassVersionHandler(NewAmassServiceFromEnv()))

	// Modular OpenVAS APIs.
	openVASService := NewOpenVASServiceFromEnv()
	mux.Handle("/openvas/version", openVASVersionHandler(openVASService))
	mux.Handle("/openvas/configs", openVASConfigsHandler(openVASService))
	mux.Handle("/openvas/targets", openVASCreateTargetHandler(openVASService))
	mux.Handle("/openvas/tasks", openVASCreateTaskHandler(openVASService))
	mux.Handle("/openvas/tasks/start", openVASStartTaskHandler(openVASService))
	mux.Handle("/openvas/tasks/status", openVASTaskStatusHandler(openVASService))
	mux.Handle("/openvas/tasks/progress", openVASTaskProgressStreamHandler(openVASService))
	mux.Handle("/openvas/reports", openVASGetReportHandler(openVASService))

	addr := ":8080"
	log.Printf("Go backend listening on %s", addr)
	if err := http.ListenAndServe(addr, mux); err != nil {
		log.Fatalf("server failed: %v", err)
	}
}
