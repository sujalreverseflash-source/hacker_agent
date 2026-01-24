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
