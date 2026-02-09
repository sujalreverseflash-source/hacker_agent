package main

import (
	"encoding/xml"
	"fmt"
	"strconv"
	"strings"
)

// parseOpenVASTaskStatus attempts to extract progress/status/report info from
// the raw <get_tasks ... details='1'/> XML response.
//
// It intentionally returns nil on parse failure so callers can still use the
// raw XML (ResponseRaw) without breaking.
func parseOpenVASTaskStatus(rawXML string, wantTaskID string) *openVASTaskStatusParsed {
	rawXML = strings.TrimSpace(rawXML)
	if rawXML == "" {
		return nil
	}

	type reportRefXML struct {
		ID string `xml:"id,attr"`
	}

	// NOTE: The gvmd XML shape can vary across versions. We keep this loose:
	// capture the most common tags and tolerate missing elements.
	type taskXML struct {
		ID       string `xml:"id,attr"`
		Name     string `xml:"name"`
		Status   string `xml:"status"`
		Progress string `xml:"progress"`

		// Different gvmd versions expose report IDs differently.
		LastReport reportRefXML   `xml:"last_report"`
		Report     reportRefXML   `xml:"report"`
		Reports    []reportRefXML `xml:"report"`
	}

	type getTasksResponseXML struct {
		Tasks []taskXML `xml:"task"`
	}

	var env getTasksResponseXML
	if err := xml.Unmarshal([]byte(rawXML), &env); err != nil {
		return nil
	}
	if len(env.Tasks) == 0 {
		return nil
	}

	// Pick the task:
	// - prefer an exact ID match
	// - fall back to the single returned task
	var t *taskXML
	wantTaskID = strings.TrimSpace(wantTaskID)
	if wantTaskID != "" {
		for i := range env.Tasks {
			if strings.TrimSpace(env.Tasks[i].ID) == wantTaskID {
				t = &env.Tasks[i]
				break
			}
		}
	}
	if t == nil {
		t = &env.Tasks[0]
	}

	status := strings.TrimSpace(t.Status)
	done := isOpenVASTaskDone(status)

	var progressPtr *float64
	var totalPtr *float64

	if p := strings.TrimSpace(t.Progress); p != "" {
		p = strings.TrimSuffix(p, "%")
		if v, err := strconv.ParseFloat(p, 64); err == nil {
			progressPtr = &v
			total := 100.0
			totalPtr = &total
		}
	}

	reportID := strings.TrimSpace(t.LastReport.ID)
	if reportID == "" {
		reportID = strings.TrimSpace(t.Report.ID)
	}
	if reportID == "" && len(t.Reports) > 0 {
		for _, r := range t.Reports {
			if strings.TrimSpace(r.ID) != "" {
				reportID = strings.TrimSpace(r.ID)
				break
			}
		}
	}

	msgParts := []string{}
	if status != "" {
		msgParts = append(msgParts, fmt.Sprintf("status=%s", status))
	}
	if progressPtr != nil {
		msgParts = append(msgParts, fmt.Sprintf("progress=%.2f%%", *progressPtr))
	}
	if reportID != "" {
		msgParts = append(msgParts, fmt.Sprintf("report_id=%s", reportID))
	}

	var message string
	if len(msgParts) > 0 {
		message = "OpenVAS task: " + strings.Join(msgParts, ", ")
	}

	return &openVASTaskStatusParsed{
		Status:   status,
		Done:     done,
		Progress: progressPtr,
		Total:    totalPtr,
		Message:  message,
		ReportID: reportID,
	}
}

func isOpenVASTaskDone(status string) bool {
	s := strings.ToLower(strings.TrimSpace(status))
	switch s {
	case "done", "stopped", "interrupted", "aborted":
		return true
	default:
		return false
	}
}

