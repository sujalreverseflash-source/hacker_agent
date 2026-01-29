package main

import (
	"encoding/json"
	"encoding/xml"
	"log"
	"net/http"
	"strings"
)

type openVASVersionResponse struct {
	VersionRaw string `json:"version_raw"`
}

// openVASConfig is the JSON representation of a single scan configuration.
type openVASConfig struct {
	ID      string `json:"id"`
	Name    string `json:"name"`
	Comment string `json:"comment,omitempty"`
}

// openVASConfigsResponse wraps all scan configurations in a stable JSON shape.
type openVASConfigsResponse struct {
	Configs []openVASConfig `json:"configs"`
}

// internal XML structs for parsing <get_configs/> output.
type openVASGetConfigsXML struct {
	Configs []openVASConfigXML `xml:"config"`
}

type openVASConfigXML struct {
	ID      string `xml:"id,attr"`
	Name    string `xml:"name"`
	Comment string `xml:"comment"`
}

// openVASCreateTargetRequest is the JSON input for creating a new target.
type openVASCreateTargetRequest struct {
	Name      string `json:"name"`
	Hosts     string `json:"hosts"`
	PortRange string `json:"port_range,omitempty"`
}

// openVASCreateTargetResponse is the JSON response returned when a target is
// created or an existing matching target is reused.
type openVASCreateTargetResponse struct {
	ID      string `json:"id"`
	Existed bool   `json:"existed,omitempty"`
}

// openVASCreateTaskRequest is the JSON input for creating a new task.
type openVASCreateTaskRequest struct {
	Name     string `json:"name"`
	ConfigID string `json:"config_id"`
	TargetID string `json:"target_id"`
}

// openVASCreateTaskResponse is the JSON response returned when a task is
// created or an existing matching task is reused.
type openVASCreateTaskResponse struct {
	ID      string `json:"id"`
	Existed bool   `json:"existed,omitempty"`
}

// openVASStartTaskRequest is the JSON input for starting an existing task.
type openVASStartTaskRequest struct {
	TaskID string `json:"task_id"`
}

// openVASStartTaskResponse wraps the raw XML response from gvmd when starting
// a task so that callers can inspect status details if needed.
type openVASStartTaskResponse struct {
	TaskID      string `json:"task_id"`
	ResponseRaw string `json:"response_raw"`
}

// openVASTaskStatusRequest is the JSON input for fetching the status/details
// of an existing task.
type openVASTaskStatusRequest struct {
	TaskID string `json:"task_id"`
}

// openVASTaskStatusResponse wraps the raw XML response from gvmd when querying
// task status so that callers can inspect status details if needed.
type openVASTaskStatusResponse struct {
	TaskID      string `json:"task_id"`
	ResponseRaw string `json:"response_raw"`
}

// openVASGetReportRequest is the JSON input for fetching a final report by ID.
type openVASGetReportRequest struct {
	ReportID string `json:"report_id"`
}

// openVASGetReportResponse wraps the raw XML response from gvmd when fetching
// a report so that callers can inspect full vulnerability details.
type openVASGetReportResponse struct {
	ReportID    string `json:"report_id"`
	ResponseRaw string `json:"response_raw"`
}

// openVASVersionHandler is a modular HTTP handler that uses OpenVASService
// to call <get_version/> and returns the raw XML in JSON.
func openVASVersionHandler(svc *OpenVASService) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodGet {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}

		versionXML, err := svc.GetVersion(r.Context())
		if err != nil {
			log.Printf("failed to get OpenVAS version: %v", err)
			http.Error(w, "failed to get OpenVAS version", http.StatusInternalServerError)
			return
		}

		w.Header().Set("Content-Type", "application/json")
		if err := json.NewEncoder(w).Encode(openVASVersionResponse{
			VersionRaw: versionXML,
		}); err != nil {
			log.Printf("failed to encode OpenVAS version response: %v", err)
		}
	})
}

// openVASConfigsHandler returns all available scan configurations (profiles)
// from OpenVAS/GVM in a simple, LLM-friendly JSON structure.
func openVASConfigsHandler(svc *OpenVASService) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodGet {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}

		configsXML, err := svc.GetConfigs(r.Context())
		if err != nil {
			log.Printf("failed to get OpenVAS configs: %v", err)
			http.Error(w, "failed to get OpenVAS configs", http.StatusInternalServerError)
			return
		}

		var parsed openVASGetConfigsXML
		if err := xml.Unmarshal([]byte(configsXML), &parsed); err != nil {
			log.Printf("failed to parse OpenVAS configs XML: %v", err)
			http.Error(w, "failed to parse OpenVAS configs", http.StatusInternalServerError)
			return
		}

		resp := openVASConfigsResponse{
			Configs: make([]openVASConfig, 0, len(parsed.Configs)),
		}

		for _, c := range parsed.Configs {
			resp.Configs = append(resp.Configs, openVASConfig{
				ID:      c.ID,
				Name:    strings.TrimSpace(c.Name),
				Comment: strings.TrimSpace(c.Comment),
			})
		}

		w.Header().Set("Content-Type", "application/json")
		if err := json.NewEncoder(w).Encode(resp); err != nil {
			log.Printf("failed to encode OpenVAS configs response: %v", err)
		}
	})
}

// openVASCreateTargetHandler creates a new OpenVAS/GVM target in an
// idempotent way. If a target with the same name and hosts already exists,
// it returns that existing target ID instead of failing.
func openVASCreateTargetHandler(svc *OpenVASService) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}

		var req openVASCreateTargetRequest
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			http.Error(w, "invalid JSON body", http.StatusBadRequest)
			return
		}

		req.Name = strings.TrimSpace(req.Name)
		req.Hosts = strings.TrimSpace(req.Hosts)
		req.PortRange = strings.TrimSpace(req.PortRange)

		if req.Name == "" || req.Hosts == "" {
			http.Error(w, "name and hosts are required", http.StatusBadRequest)
			return
		}

		id, existed, err := svc.CreateTarget(r.Context(), req.Name, req.Hosts, req.PortRange)
		if err != nil {
			log.Printf("failed to create OpenVAS target: %v", err)
			http.Error(w, "failed to create OpenVAS target", http.StatusInternalServerError)
			return
		}

		w.Header().Set("Content-Type", "application/json")
		if err := json.NewEncoder(w).Encode(openVASCreateTargetResponse{
			ID:      id,
			Existed: existed,
		}); err != nil {
			log.Printf("failed to encode OpenVAS create target response: %v", err)
		}
	})
}

// openVASCreateTaskHandler creates a new OpenVAS/GVM task in an idempotent
// way. If a task with the same name, config ID and target ID already exists,
// it returns that existing task ID instead of failing.
func openVASCreateTaskHandler(svc *OpenVASService) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}

		var req openVASCreateTaskRequest
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			http.Error(w, "invalid JSON body", http.StatusBadRequest)
			return
		}

		req.Name = strings.TrimSpace(req.Name)
		req.ConfigID = strings.TrimSpace(req.ConfigID)
		req.TargetID = strings.TrimSpace(req.TargetID)

		if req.Name == "" || req.ConfigID == "" || req.TargetID == "" {
			http.Error(w, "name, config_id and target_id are required", http.StatusBadRequest)
			return
		}

		id, existed, err := svc.CreateTask(r.Context(), req.Name, req.ConfigID, req.TargetID)
		if err != nil {
			log.Printf("failed to create OpenVAS task: %v", err)
			http.Error(w, "failed to create OpenVAS task", http.StatusInternalServerError)
			return
		}

		w.Header().Set("Content-Type", "application/json")
		if err := json.NewEncoder(w).Encode(openVASCreateTaskResponse{
			ID:      id,
			Existed: existed,
		}); err != nil {
			log.Printf("failed to encode OpenVAS create task response: %v", err)
		}
	})
}

// openVASStartTaskHandler starts an existing OpenVAS/GVM task by ID.
func openVASStartTaskHandler(svc *OpenVASService) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}

		var req openVASStartTaskRequest
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			http.Error(w, "invalid JSON body", http.StatusBadRequest)
			return
		}

		req.TaskID = strings.TrimSpace(req.TaskID)
		if req.TaskID == "" {
			http.Error(w, "task_id is required", http.StatusBadRequest)
			return
		}

		raw, err := svc.StartTask(r.Context(), req.TaskID)
		if err != nil {
			log.Printf("failed to start OpenVAS task: %v", err)
			http.Error(w, "failed to start OpenVAS task", http.StatusInternalServerError)
			return
		}

		w.Header().Set("Content-Type", "application/json")
		if err := json.NewEncoder(w).Encode(openVASStartTaskResponse{
			TaskID:      req.TaskID,
			ResponseRaw: raw,
		}); err != nil {
			log.Printf("failed to encode OpenVAS start task response: %v", err)
		}
	})
}

// openVASTaskStatusHandler fetches the current status/details for an existing
// OpenVAS/GVM task by ID.
func openVASTaskStatusHandler(svc *OpenVASService) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}

		var req openVASTaskStatusRequest
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			http.Error(w, "invalid JSON body", http.StatusBadRequest)
			return
		}

		req.TaskID = strings.TrimSpace(req.TaskID)
		if req.TaskID == "" {
			http.Error(w, "task_id is required", http.StatusBadRequest)
			return
		}

		raw, err := svc.GetTaskStatus(r.Context(), req.TaskID)
		if err != nil {
			log.Printf("failed to get OpenVAS task status: %v", err)
			http.Error(w, "failed to get OpenVAS task status", http.StatusInternalServerError)
			return
		}

		w.Header().Set("Content-Type", "application/json")
		if err := json.NewEncoder(w).Encode(openVASTaskStatusResponse{
			TaskID:      req.TaskID,
			ResponseRaw: raw,
		}); err != nil {
			log.Printf("failed to encode OpenVAS task status response: %v", err)
		}
	})
}

// openVASGetReportHandler fetches the final report for a given report ID.
func openVASGetReportHandler(svc *OpenVASService) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}

		var req openVASGetReportRequest
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			http.Error(w, "invalid JSON body", http.StatusBadRequest)
			return
		}

		req.ReportID = strings.TrimSpace(req.ReportID)
		if req.ReportID == "" {
			http.Error(w, "report_id is required", http.StatusBadRequest)
			return
		}

		raw, err := svc.GetReport(r.Context(), req.ReportID)
		if err != nil {
			log.Printf("failed to get OpenVAS report: %v", err)
			http.Error(w, "failed to get OpenVAS report", http.StatusInternalServerError)
			return
		}

		w.Header().Set("Content-Type", "application/json")
		if err := json.NewEncoder(w).Encode(openVASGetReportResponse{
			ReportID:    req.ReportID,
			ResponseRaw: raw,
		}); err != nil {
			log.Printf("failed to encode OpenVAS get report response: %v", err)
		}
	})
}
