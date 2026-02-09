package main

import (
	"fmt"
)

// buildNmapArgs converts the incoming scanRequest into a safe list of nmap CLI args.
// It shares the same validation behavior as the existing handler.
func buildNmapArgs(req scanRequest) ([]string, error) {
	var cmdArgs []string

	// Add timing template
	timingTemplate := req.Timing
	if timingTemplate == "" {
		timingTemplate = "T2"
	}
	validTimings := map[string]bool{"T0": true, "T1": true, "T2": true, "T3": true, "T4": true, "T5": true}
	if !validTimings[timingTemplate] {
		return nil, fmt.Errorf("invalid timing template. Must be one of: T0, T1, T2, T3, T4, T5")
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

	// Add output format (NOTE: this flag usually expects an output file; we keep
	// the current behavior for backward compatibility even if nmap errors).
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

	// Add target (required)
	cmdArgs = append(cmdArgs, req.Target)

	return cmdArgs, nil
}

