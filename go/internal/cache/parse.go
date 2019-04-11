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
