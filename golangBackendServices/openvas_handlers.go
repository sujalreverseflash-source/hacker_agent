package main

import (
	"encoding/json"
	"log"
	"net/http"
)

type openVASVersionResponse struct {
	VersionRaw string `json:"version_raw"`
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

