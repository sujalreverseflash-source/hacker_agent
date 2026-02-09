package main

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"regexp"
	"strings"
	"time"
)

type amassEnumRequest struct {
	// Required
	Domain string `json:"domain"`

	// Optional
	Mode                  string `json:"mode,omitempty"`          // "passive" (default) or "active"
	OutputFormat          string `json:"output_format,omitempty"` // "txt" (default) or "json"
	SaveToFile            bool   `json:"save_to_file,omitempty"`
	OutputFile            string `json:"output_file,omitempty"` // optional filename (no directories). If SaveToFile=false, ignored.
	ContextTimeoutSeconds int    `json:"context_timeout_seconds,omitempty"`
}

type amassEnumResponse struct {
	Domain       string `json:"domain"`
	Mode         string `json:"mode"`
	OutputFormat string `json:"output_format"`

	Saved      bool   `json:"saved,omitempty"`
	OutputFile string `json:"output_file,omitempty"`

	RawOutput string `json:"raw_output,omitempty"`
	Stderr    string `json:"stderr,omitempty"`
}

type amassVersionResponse struct {
	VersionRaw string `json:"version_raw"`
}

func amassVersionHandler(svc *AmassService) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodGet {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}

		version, err := svc.GetVersion(r.Context())
		if err != nil {
			log.Printf("failed to get Amass version: %v", err)
			http.Error(w, "failed to get Amass version", http.StatusInternalServerError)
			return
		}

		w.Header().Set("Content-Type", "application/json")
		if err := json.NewEncoder(w).Encode(amassVersionResponse{VersionRaw: version}); err != nil {
			log.Printf("failed to encode Amass version response: %v", err)
		}
	})
}

var safeFilenameRe = regexp.MustCompile(`[^a-zA-Z0-9._-]+`)

func sanitizeFilename(name string) string {
	name = strings.TrimSpace(name)
	name = filepath.Base(name)
	if name == "." || name == "/" {
		return ""
	}
	name = safeFilenameRe.ReplaceAllString(name, "_")
	name = strings.Trim(name, "._-")
	return name
}

func ensureExt(name, ext string) string {
	ext = strings.ToLower(strings.TrimSpace(ext))
	if ext == "" || ext[0] != '.' {
		return name
	}
	if strings.HasSuffix(strings.ToLower(name), ext) {
		return name
	}
	return name + ext
}

func readFileIfExists(path string) (string, error) {
	b, err := os.ReadFile(path)
	if err != nil {
		return "", err
	}
	return string(b), nil
}

func amassEnumHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
		return
	}

	var req amassEnumRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "invalid JSON body", http.StatusBadRequest)
		return
	}

	req.Domain = strings.TrimSpace(req.Domain)
	if req.Domain == "" {
		http.Error(w, "domain is required", http.StatusBadRequest)
		return
	}

	mode := strings.ToLower(strings.TrimSpace(req.Mode))
	if mode == "" {
		mode = "passive"
	}
	if mode != "passive" && mode != "active" {
		http.Error(w, `invalid mode. Must be "passive" or "active"`, http.StatusBadRequest)
		return
	}

	format := strings.ToLower(strings.TrimSpace(req.OutputFormat))
	if format == "" {
		format = "txt"
	}
	if format != "txt" && format != "json" {
		http.Error(w, `invalid output_format. Must be "txt" or "json"`, http.StatusBadRequest)
		return
	}

	timeoutSeconds := req.ContextTimeoutSeconds
	if timeoutSeconds <= 0 {
		timeoutSeconds = 1800
	}
	ctx, cancel := context.WithTimeout(r.Context(), time.Duration(timeoutSeconds)*time.Second)
	defer cancel()

	// Build args
	args := []string{"enum", "-d", req.Domain}
	if mode == "active" {
		args = append(args, "-active")
	}

	// Output directory for saved (or temp) files
	outDir := filepath.Join(".", "outputs")
	if err := os.MkdirAll(outDir, 0o755); err != nil {
		log.Printf("failed to create outputs dir: %v", err)
		http.Error(w, "failed to prepare output directory", http.StatusInternalServerError)
		return
	}

	// For JSON, Amass writes to a file via -json (no stdout JSON support).
	// For TXT, we can return stdout, and optionally also write via -o.
	var outPath string
	var shouldDeleteAfter bool

	if format == "json" || req.SaveToFile {
		base := sanitizeFilename(req.OutputFile)
		if base == "" {
			base = fmt.Sprintf("amass_%s_%d", sanitizeFilename(req.Domain), time.Now().Unix())
		}
		if format == "json" {
			base = ensureExt(base, ".json")
		} else {
			base = ensureExt(base, ".txt")
		}
		outPath = filepath.Join(outDir, base)
		if !req.SaveToFile {
			shouldDeleteAfter = true
		}
	}

	if format == "json" {
		args = append(args, "-json", outPath)
	} else if req.SaveToFile {
		args = append(args, "-o", outPath)
	}

	svc := NewAmassServiceFromEnv()
	cmd := exec.CommandContext(ctx, svc.Bin, args...)

	var stdout bytes.Buffer
	var stderr bytes.Buffer
	cmd.Stdout = &stdout
	cmd.Stderr = &stderr

	runErr := cmd.Run()

	// Try to load output from file if we used one.
	var fileContent string
	if outPath != "" {
		if s, err := readFileIfExists(outPath); err == nil {
			fileContent = s
		} else {
			// If Amass failed, the file might not exist; keep going.
			if runErr == nil {
				log.Printf("failed to read amass output file %s: %v", outPath, err)
			}
		}
	}

	if shouldDeleteAfter && outPath != "" {
		_ = os.Remove(outPath)
	}

	resp := amassEnumResponse{
		Domain:       req.Domain,
		Mode:         mode,
		OutputFormat: format,
		Stderr:       strings.TrimSpace(stderr.String()),
	}

	if req.SaveToFile && outPath != "" {
		resp.Saved = true
		resp.OutputFile = outPath
	}

	// Prefer file content when we wrote to a file; otherwise return stdout.
	if strings.TrimSpace(fileContent) != "" {
		resp.RawOutput = fileContent
	} else {
		resp.RawOutput = strings.TrimSpace(stdout.String())
	}

	if runErr != nil {
		// Still return whatever we have for debugging.
		log.Printf("amass enum error for %s: %v (stderr=%s)", req.Domain, runErr, strings.TrimSpace(stderr.String()))
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusInternalServerError)
		_ = json.NewEncoder(w).Encode(resp)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	if err := json.NewEncoder(w).Encode(resp); err != nil {
		log.Printf("failed to encode amass enum response: %v", err)
	}
}
