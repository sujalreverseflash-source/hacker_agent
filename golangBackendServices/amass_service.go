package main

import (
	"context"
	"fmt"
	"os"
	"os/exec"
	"strings"
)

// AmassService encapsulates calls to the `amass` CLI.
type AmassService struct {
	Bin string
}

// NewAmassServiceFromEnv builds a service using environment variables.
//
// Optional:
//   - AMASS_BIN (default: "amass")
func NewAmassServiceFromEnv() *AmassService {
	bin := strings.TrimSpace(os.Getenv("AMASS_BIN"))
	if bin == "" {
		bin = "amass"
	}
	return &AmassService{Bin: bin}
}

func (s *AmassService) GetVersion(ctx context.Context) (string, error) {
	cmd := exec.CommandContext(ctx, s.Bin, "version")
	out, err := cmd.CombinedOutput()
	if err != nil {
		return "", fmt.Errorf("amass version failed: %w; output: %s", err, string(out))
	}
	return string(out), nil
}
