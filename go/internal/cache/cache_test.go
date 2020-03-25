package cache

import (
	"os"
	"path/filepath"
	"testing"
)

// These vectors MUST match the Rust engine's known_vectors test in
// rust/src/hash.rs, guaranteeing the two tools produce identical digests.
func TestKnownVectors(t *testing.T) {
	cases := map[string]string{
		"":       "cbf29ce484222325",
