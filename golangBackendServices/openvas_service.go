package main

import (
	"context"
	"fmt"
	"os"
	"os/exec"
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
