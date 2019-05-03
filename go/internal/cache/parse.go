package cache

import (
	"bytes"
	"encoding/json"
	"fmt"
)

// wireManifest mirrors the JSON structure for decoding. We decode with the
// standard library, then convert to the strongly typed Manifest. Encoding is
// done by hand (ToJSON) to guarantee canonical byte output.
type wireManifest struct {
	Version  uint64      `json:"version"`
	Producer string      `json:"producer"`
	Key      string      `json:"key"`
	Command  []string    `json:"command"`
	Inputs   []wireEntry `json:"inputs"`
	Outputs  []wireEntry `json:"outputs"`
}

type wireEntry struct {
	Path   string `json:"path"`
	Digest string `json:"digest"`
	Size   uint64 `json:"size"`
}

// ParseManifest decodes a manifest from JSON produced by either implementation.
func ParseManifest(data []byte) (*Manifest, error) {
	var w wireManifest
	dec := json.NewDecoder(bytes.NewReader(data))
	dec.DisallowUnknownFields()
	if err := dec.Decode(&w); err != nil {
		return nil, fmt.Errorf("decoding manifest: %w", err)
	}
	m := &Manifest{
		Version:  w.Version,
		Producer: w.Producer,
		Key:      w.Key,
		Command:  w.Command,
		Inputs:   convertEntries(w.Inputs),
		Outputs:  convertEntries(w.Outputs),
	}
	if m.Command == nil {
		m.Command = []string{}
	}
	return m, nil
