package main

import (
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"strconv"
	"strings"
	"time"
)

// openVASTaskProgressStreamHandler streams task progress updates via SSE.
//
// Endpoint:
//   - GET /openvas/tasks/progress?task_id=...&interval_seconds=2
//
// It polls gvmd using the existing GetTaskStatus API, parses out status/progress,
// then emits SSE events until the task is done or the client disconnects.
func openVASTaskProgressStreamHandler(svc *OpenVASService) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodGet {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}

		taskID := strings.TrimSpace(r.URL.Query().Get("task_id"))
		if taskID == "" {
			http.Error(w, "task_id is required", http.StatusBadRequest)
			return
		}

		interval := 2 * time.Second
		if s := strings.TrimSpace(r.URL.Query().Get("interval_seconds")); s != "" {
			if v, err := strconv.Atoi(s); err == nil && v > 0 && v <= 60 {
				interval = time.Duration(v) * time.Second
			}
		}

		flusher, ok := w.(http.Flusher)
		if !ok {
			http.Error(w, "streaming unsupported", http.StatusInternalServerError)
			return
		}

		w.Header().Set("Content-Type", "text/event-stream")
		w.Header().Set("Cache-Control", "no-cache")
		w.Header().Set("Connection", "keep-alive")

		// Initial event so clients know the stream is alive.
		fmt.Fprintf(w, "event: ready\ndata: %s\n\n", mustJSON(map[string]any{
			"task_id":           taskID,
			"interval_seconds":  int(interval.Seconds()),
			"server_timestamp":  time.Now().UTC().Format(time.RFC3339Nano),
			"message":           "progress stream started",
			"content_type_hint": "application/json",
		}))
		flusher.Flush()

		ticker := time.NewTicker(interval)
		defer ticker.Stop()

		ctx := r.Context()
		var lastHash string

		for {
			select {
			case <-ctx.Done():
				return
			case <-ticker.C:
				raw, err := svc.GetTaskStatus(ctx, taskID)
				if err != nil {
					log.Printf("openvas progress stream: task_id=%s get status failed: %v", taskID, err)
					fmt.Fprintf(w, "event: error\ndata: %s\n\n", mustJSON(map[string]any{
						"task_id": taskID,
						"error":   "failed to fetch task status",
					}))
					flusher.Flush()
					continue
				}

				parsed := parseOpenVASTaskStatus(raw, taskID)
				payload := map[string]any{
					"task_id":          taskID,
					"server_timestamp": time.Now().UTC().Format(time.RFC3339Nano),
					"parsed":           parsed,
				}

				// Avoid flooding: only emit when something changed meaningfully.
				b, _ := json.Marshal(payload)
				currHash := string(b)
				if currHash == lastHash {
					continue
				}
				lastHash = currHash

				fmt.Fprintf(w, "event: progress\ndata: %s\n\n", string(b))
				flusher.Flush()

				if parsed != nil && parsed.Done {
					fmt.Fprintf(w, "event: done\ndata: %s\n\n", mustJSON(map[string]any{
						"task_id": taskID,
						"message": "task completed",
						"parsed":  parsed,
					}))
					flusher.Flush()
					return
				}
			}
		}
	})
}

func mustJSON(v any) string {
	b, err := json.Marshal(v)
	if err != nil {
		// Best-effort fallback; SSE should still remain valid.
		return `{"error":"failed to marshal json"}`
	}
	return string(b)
}

