package main

import (
	"context"
	"encoding/xml"
	"fmt"
	"os"
	"os/exec"
	"strings"
)

// OpenVASService encapsulates calls to gvm-cli (OpenVAS/GVM).
// We start with just GetVersion to keep the design modular and focused.
type OpenVASService struct {
	ContainerName string
	Username      string
	Password      string
	Host          string
	Port          string
}

// NewOpenVASServiceFromEnv builds a service using environment variables.
//
// Required:
//   - GVM_PASSWORD
//
// Optional (with defaults):
//   - OPENVAS_CONTAINER_NAME (default: "openvas")
//   - GVM_USERNAME          (default: "admin")
//   - GVM_HOST              (default: "127.0.0.1")
//   - GVM_PORT              (default: "9390")
func NewOpenVASServiceFromEnv() *OpenVASService {
	container := os.Getenv("OPENVAS_CONTAINER_NAME")
	if container == "" {
		container = "openvas"
	}

	username := os.Getenv("GVM_USERNAME")
	if username == "" {
		username = "admin"
	}

	password := os.Getenv("GVM_PASSWORD")

	host := os.Getenv("GVM_HOST")
	if host == "" {
		host = "127.0.0.1"
	}

	port := os.Getenv("GVM_PORT")
	if port == "" {
		port = "9390"
	}

	return &OpenVASService{
		ContainerName: container,
		Username:      username,
		Password:      password,
		Host:          host,
		Port:          port,
	}
}

// GetVersion calls gvm-cli with <get_version/> and returns the raw XML
// response from gvmd.
func (s *OpenVASService) GetVersion(ctx context.Context) (string, error) {
	if s.Password == "" {
		return "", fmt.Errorf("GVM_PASSWORD is not set")
	}

	args := []string{
		"exec",
		"-u", "gvm",
		s.ContainerName,
		"gvm-cli",
		"--gmp-username", s.Username,
		"--gmp-password", s.Password,
		"tls",
		"--hostname", s.Host,
		"--port", s.Port,
		"--xml", "<get_version/>",
	}

	cmd := exec.CommandContext(ctx, "docker", args...)
	out, err := cmd.CombinedOutput()
	if err != nil {
		return "", fmt.Errorf("gvm-cli get_version failed: %w; output: %s", err, string(out))
	}

	return string(out), nil
}

// GetConfigs calls gvm-cli with <get_configs/> and returns the raw XML
// response listing all available scan configurations.
func (s *OpenVASService) GetConfigs(ctx context.Context) (string, error) {
	if s.Password == "" {
		return "", fmt.Errorf("GVM_PASSWORD is not set")
	}

	args := []string{
		"exec",
		"-u", "gvm",
		s.ContainerName,
		"gvm-cli",
		"--gmp-username", s.Username,
		"--gmp-password", s.Password,
		"tls",
		"--hostname", s.Host,
		"--port", s.Port,
		"--xml", "<get_configs/>",
	}

	cmd := exec.CommandContext(ctx, "docker", args...)
	out, err := cmd.CombinedOutput()
	if err != nil {
		return "", fmt.Errorf("gvm-cli get_configs failed: %w; output: %s", err, string(out))
	}

	return string(out), nil
}

// internal XML structs for working with targets.
type openVASTargetsXML struct {
	Targets []openVASTargetXML `xml:"target"`
}

type openVASTargetXML struct {
	ID       string `xml:"id,attr"`
	Name     string `xml:"name"`
	HostsRaw string `xml:"hosts"`
}

// CreateTarget ensures idempotent target creation:
//   - If a target with the same name and hosts already exists, it returns
//     the existing target ID and existed=true.
//   - Otherwise it creates a new target via <create_target> and returns
//     the new target ID and existed=false.
func (s *OpenVASService) CreateTarget(ctx context.Context, name, hosts, portRange string) (id string, existed bool, err error) {
	if s.Password == "" {
		return "", false, fmt.Errorf("GVM_PASSWORD is not set")
	}

	name = strings.TrimSpace(name)
	hosts = strings.TrimSpace(hosts)
	portRange = strings.TrimSpace(portRange)

	if name == "" || hosts == "" {
		return "", false, fmt.Errorf("name and hosts are required")
	}

	// First: check for an existing target with the same name and hosts.
	getTargetsArgs := []string{
		"exec",
		"-u", "gvm",
		s.ContainerName,
		"gvm-cli",
		"--gmp-username", s.Username,
		"--gmp-password", s.Password,
		"tls",
		"--hostname", s.Host,
		"--port", s.Port,
		"--xml", "<get_targets/>",
	}

	getTargetsCmd := exec.CommandContext(ctx, "docker", getTargetsArgs...)
	targetsOut, getTargetsErr := getTargetsCmd.CombinedOutput()
	if getTargetsErr == nil {
		var parsed openVASTargetsXML
		if err := xml.Unmarshal(targetsOut, &parsed); err == nil {
			wantName := strings.TrimSpace(name)
			wantHosts := strings.TrimSpace(hosts)

			for _, t := range parsed.Targets {
				if strings.TrimSpace(t.Name) != wantName {
					continue
				}

				// Hosts can be represented in different ways in XML; perform
				// a relaxed match against the raw hosts XML/text.
				hostsLower := strings.ToLower(wantHosts)
				rawLower := strings.ToLower(t.HostsRaw)
				if hostsLower != "" && strings.Contains(rawLower, hostsLower) {
					return t.ID, true, nil
				}
			}
		}
	}

	// If we didn't find an existing target (or get_targets failed), create one.
	type createTargetXML struct {
		XMLName   xml.Name `xml:"create_target"`
		Name      string   `xml:"name"`
		Hosts     string   `xml:"hosts"`
		PortRange string   `xml:"port_range,omitempty"`
	}

	payload := createTargetXML{
		Name:  name,
		Hosts: hosts,
	}
	if portRange != "" {
		payload.PortRange = portRange
	}

	xmlBody, err := xml.Marshal(&payload)
	if err != nil {
		return "", false, fmt.Errorf("failed to marshal create_target XML: %w", err)
	}

	createArgs := []string{
		"exec",
		"-u", "gvm",
		s.ContainerName,
		"gvm-cli",
		"--gmp-username", s.Username,
		"--gmp-password", s.Password,
		"tls",
		"--hostname", s.Host,
		"--port", s.Port,
		"--xml", string(xmlBody),
	}

	createCmd := exec.CommandContext(ctx, "docker", createArgs...)
	createOut, createErr := createCmd.CombinedOutput()
	if createErr != nil {
		return "", false, fmt.Errorf("gvm-cli create_target failed: %w; output: %s", createErr, string(createOut))
	}

	type createTargetResponseXML struct {
		XMLName xml.Name `xml:"create_target_response"`
		ID      string   `xml:"id,attr"`
	}

	var resp createTargetResponseXML
	if err := xml.Unmarshal(createOut, &resp); err != nil {
		return "", false, fmt.Errorf("failed to parse create_target_response XML: %w; output: %s", err, string(createOut))
	}
	if strings.TrimSpace(resp.ID) == "" {
		return "", false, fmt.Errorf("empty target id in create_target_response; output: %s", string(createOut))
	}

	return strings.TrimSpace(resp.ID), false, nil
}

// internal XML structs for working with tasks.
type openVASTasksXML struct {
	Tasks []openVASTaskXML `xml:"task"`
}

type openVASTaskXML struct {
	ID     string               `xml:"id,attr"`
	Name   string               `xml:"name"`
	Config openVASTaskConfigXML `xml:"config"`
	Target openVASTaskTargetXML `xml:"target"`
}

type openVASTaskConfigXML struct {
	ID string `xml:"id,attr"`
}

type openVASTaskTargetXML struct {
	ID string `xml:"id,attr"`
}

// CreateTask ensures idempotent task creation:
//   - If a task with the same name, config ID, and target ID already exists,
//     it returns the existing task ID and existed=true.
//   - Otherwise it creates a new task via <create_task> and returns the new
//     task ID and existed=false.
func (s *OpenVASService) CreateTask(ctx context.Context, name, configID, targetID string) (id string, existed bool, err error) {
	if s.Password == "" {
		return "", false, fmt.Errorf("GVM_PASSWORD is not set")
	}

	name = strings.TrimSpace(name)
	configID = strings.TrimSpace(configID)
	targetID = strings.TrimSpace(targetID)

	if name == "" || configID == "" || targetID == "" {
		return "", false, fmt.Errorf("name, configID, and targetID are required")
	}

	// First: check for an existing task with the same name, config, and target.
	getTasksArgs := []string{
		"exec",
		"-u", "gvm",
		s.ContainerName,
		"gvm-cli",
		"--gmp-username", s.Username,
		"--gmp-password", s.Password,
		"tls",
		"--hostname", s.Host,
		"--port", s.Port,
		"--xml", "<get_tasks/>",
	}

	getTasksCmd := exec.CommandContext(ctx, "docker", getTasksArgs...)
	tasksOut, getTasksErr := getTasksCmd.CombinedOutput()
	if getTasksErr == nil {
		var parsed openVASTasksXML
		if err := xml.Unmarshal(tasksOut, &parsed); err == nil {
			wantName := strings.TrimSpace(name)
			wantConfig := strings.TrimSpace(configID)
			wantTarget := strings.TrimSpace(targetID)

			for _, t := range parsed.Tasks {
				if strings.TrimSpace(t.Name) != wantName {
					continue
				}
				if strings.TrimSpace(t.Config.ID) != wantConfig {
					continue
				}
				if strings.TrimSpace(t.Target.ID) != wantTarget {
					continue
				}
				return t.ID, true, nil
			}
		}
	}

	// If we didn't find an existing task (or get_tasks failed), create one.
	type createTaskXML struct {
		XMLName xml.Name             `xml:"create_task"`
		Name    string               `xml:"name"`
		Config  openVASTaskConfigXML `xml:"config"`
		Target  openVASTaskTargetXML `xml:"target"`
	}

	payload := createTaskXML{
		Name: name,
		Config: openVASTaskConfigXML{
			ID: configID,
		},
		Target: openVASTaskTargetXML{
			ID: targetID,
		},
	}

	xmlBody, err := xml.Marshal(&payload)
	if err != nil {
		return "", false, fmt.Errorf("failed to marshal create_task XML: %w", err)
	}

	createArgs := []string{
		"exec",
		"-u", "gvm",
		s.ContainerName,
		"gvm-cli",
		"--gmp-username", s.Username,
		"--gmp-password", s.Password,
		"tls",
		"--hostname", s.Host,
		"--port", s.Port,
		"--xml", string(xmlBody),
	}

	createCmd := exec.CommandContext(ctx, "docker", createArgs...)
	createOut, createErr := createCmd.CombinedOutput()
	if createErr != nil {
		return "", false, fmt.Errorf("gvm-cli create_task failed: %w; output: %s", createErr, string(createOut))
	}

	type createTaskResponseXML struct {
		XMLName xml.Name `xml:"create_task_response"`
		ID      string   `xml:"id,attr"`
	}

	var resp createTaskResponseXML
	if err := xml.Unmarshal(createOut, &resp); err != nil {
		return "", false, fmt.Errorf("failed to parse create_task_response XML: %w; output: %s", err, string(createOut))
	}
	if strings.TrimSpace(resp.ID) == "" {
		return "", false, fmt.Errorf("empty task id in create_task_response; output: %s", string(createOut))
	}

	return strings.TrimSpace(resp.ID), false, nil
}

// StartTask starts an existing OpenVAS/GVM task by ID and returns the raw XML
// response from gvmd. Callers can inspect the XML for status details.
func (s *OpenVASService) StartTask(ctx context.Context, taskID string) (string, error) {
	if s.Password == "" {
		return "", fmt.Errorf("GVM_PASSWORD is not set")
	}

	taskID = strings.TrimSpace(taskID)
	if taskID == "" {
		return "", fmt.Errorf("taskID is required")
	}

	xmlBody := fmt.Sprintf("<start_task task_id='%s'/>", taskID)

	args := []string{
		"exec",
		"-u", "gvm",
		s.ContainerName,
		"gvm-cli",
		"--gmp-username", s.Username,
		"--gmp-password", s.Password,
		"tls",
		"--hostname", s.Host,
		"--port", s.Port,
		"--xml", xmlBody,
	}

	cmd := exec.CommandContext(ctx, "docker", args...)
	out, err := cmd.CombinedOutput()
	if err != nil {
		return "", fmt.Errorf("gvm-cli start_task failed: %w; output: %s", err, string(out))
	}

	return string(out), nil
}

// GetTaskStatus fetches the current status/details for an existing OpenVAS/GVM
// task by ID using <get_tasks task_id='...' details='1'/> and returns the raw
// XML response from gvmd.
func (s *OpenVASService) GetTaskStatus(ctx context.Context, taskID string) (string, error) {
	if s.Password == "" {
		return "", fmt.Errorf("GVM_PASSWORD is not set")
	}

	taskID = strings.TrimSpace(taskID)
	if taskID == "" {
		return "", fmt.Errorf("taskID is required")
	}

	xmlBody := fmt.Sprintf("<get_tasks task_id='%s' details='1'/>", taskID)

	args := []string{
		"exec",
		"-u", "gvm",
		s.ContainerName,
		"gvm-cli",
		"--gmp-username", s.Username,
		"--gmp-password", s.Password,
		"tls",
		"--hostname", s.Host,
		"--port", s.Port,
		"--xml", xmlBody,
	}

	cmd := exec.CommandContext(ctx, "docker", args...)
	out, err := cmd.CombinedOutput()
	if err != nil {
		return "", fmt.Errorf("gvm-cli get_tasks failed: %w; output: %s", err, string(out))
	}

	return string(out), nil
}

// GetReport fetches the final report for a given report ID using
// <get_reports report_id='...' details='1'/> and returns the raw XML
// response from gvmd.
func (s *OpenVASService) GetReport(ctx context.Context, reportID string) (string, error) {
	if s.Password == "" {
		return "", fmt.Errorf("GVM_PASSWORD is not set")
	}

	reportID = strings.TrimSpace(reportID)
	if reportID == "" {
		return "", fmt.Errorf("reportID is required")
	}

	xmlBody := fmt.Sprintf("<get_reports report_id='%s' details='1'/>", reportID)

	args := []string{
		"exec",
		"-u", "gvm",
		s.ContainerName,
		"gvm-cli",
		"--gmp-username", s.Username,
		"--gmp-password", s.Password,
		"tls",
		"--hostname", s.Host,
		"--port", s.Port,
		"--xml", xmlBody,
	}

	cmd := exec.CommandContext(ctx, "docker", args...)
	out, err := cmd.CombinedOutput()
	if err != nil {
		return "", fmt.Errorf("gvm-cli get_reports failed: %w; output: %s", err, string(out))
	}

	return string(out), nil
}
