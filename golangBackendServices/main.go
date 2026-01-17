package main

import (
	"encoding/json"
	"log"
	"net/http"
	"os/exec"
	"strings"
)

type scanRequest struct {
	Target string `json:"target"`
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

	// By default use Nmap timing template T2 (Polite) unless explicitly changed.
	// Example: nmap -T2 <target>
	cmd := exec.Command("nmap", "-T2", req.Target)
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
	mux := http.NewServeMux()
	mux.HandleFunc("/scan-open-ports", scanOpenPortsHandler)

	addr := ":8080"
	log.Printf("Go backend listening on %s", addr)
	if err := http.ListenAndServe(addr, mux); err != nil {
		log.Fatalf("server failed: %v", err)
	}
}
