package main

import (
	"bufio"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"net/http"
	"os/exec"
	"strings"
	"sync"
	"time"
)

type nmapSSELine struct {
	Stream    string `json:"stream"` // "stdout" | "stderr"
	Line      string `json:"line"`
	Timestamp string `json:"timestamp"`
}

// scanOpenPortsStreamHandler runs nmap and streams its output via SSE.
//
// Method: POST
// URL:    /scan-open-ports/stream
// Body:   same JSON as /scan-open-ports
//
// SSE events:
// - ready:    stream started + args
// - output:   one line from stdout/stderr
// - error:    execution error
// - done:     process finished
func scanOpenPortsStreamHandler(w http.ResponseWriter, r *http.Request) {
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

	args, err := buildNmapArgs(req)
	if err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	flusher, ok := w.(http.Flusher)
	if !ok {
		http.Error(w, "streaming unsupported", http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "text/event-stream")
	w.Header().Set("Cache-Control", "no-cache")
	w.Header().Set("Connection", "keep-alive")

	ctx := r.Context()

	// Start the command.
	cmd := exec.CommandContext(ctx, "nmap", args...)
	stdout, err := cmd.StdoutPipe()
	if err != nil {
		http.Error(w, "failed to get stdout pipe", http.StatusInternalServerError)
		return
	}
	stderr, err := cmd.StderrPipe()
	if err != nil {
		http.Error(w, "failed to get stderr pipe", http.StatusInternalServerError)
		return
	}

	if err := cmd.Start(); err != nil {
		http.Error(w, "failed to start nmap", http.StatusInternalServerError)
		return
	}

	// Initial ready event.
	fmt.Fprintf(w, "event: ready\ndata: %s\n\n", mustJSON(map[string]any{
		"target":           req.Target,
		"args":             args,
		"server_timestamp": time.Now().UTC().Format(time.RFC3339Nano),
		"message":          "nmap stream started",
	}))
	flusher.Flush()

	lines := make(chan nmapSSELine, 256)
	var wg sync.WaitGroup
	wg.Add(2)

	go streamReader(ctx, "stdout", stdout, lines, &wg)
	go streamReader(ctx, "stderr", stderr, lines, &wg)

	// Close channel when both readers finish.
	go func() {
		wg.Wait()
		close(lines)
	}()

	// Keepalive ticker (SSE proxies can time out otherwise).
	keepAlive := time.NewTicker(10 * time.Second)
	defer keepAlive.Stop()

	// Forward lines.
	for {
		select {
		case <-ctx.Done():
			// CommandContext will terminate the process.
			return
		case <-keepAlive.C:
			// SSE comment line.
			io.WriteString(w, ": keep-alive\n\n")
			flusher.Flush()
		case l, ok := <-lines:
			if !ok {
				// Readers drained; wait for process exit.
				waitErr := cmd.Wait()
				if waitErr != nil {
					fmt.Fprintf(w, "event: error\ndata: %s\n\n", mustJSON(map[string]any{
						"target": req.Target,
						"error":  waitErr.Error(),
					}))
					flusher.Flush()
					return
				}

				fmt.Fprintf(w, "event: done\ndata: %s\n\n", mustJSON(map[string]any{
					"target":           req.Target,
					"exit":             "ok",
					"server_timestamp": time.Now().UTC().Format(time.RFC3339Nano),
					"message":          "nmap completed",
				}))
				flusher.Flush()
				return
			}

			b, _ := json.Marshal(l)
			fmt.Fprintf(w, "event: output\ndata: %s\n\n", string(b))
			flusher.Flush()
		}
	}
}

func streamReader(ctx context.Context, stream string, r io.Reader, out chan<- nmapSSELine, wg *sync.WaitGroup) {
	defer wg.Done()

	sc := bufio.NewScanner(r)
	// nmap output lines can be long; increase scanner buffer.
	buf := make([]byte, 0, 64*1024)
	sc.Buffer(buf, 1024*1024)

	for sc.Scan() {
		select {
		case <-ctx.Done():
			return
		default:
		}

		out <- nmapSSELine{
			Stream:    stream,
			Line:      sc.Text(),
			Timestamp: time.Now().UTC().Format(time.RFC3339Nano),
		}
	}
	if err := sc.Err(); err != nil && ctx.Err() == nil {
		log.Printf("nmap stream reader error (%s): %v", stream, err)
	}
}

