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
		"a":      "af63dc4c8601ec8c",
		"foobar": "85944171f73967e8",
	}
	for input, want := range cases {
		if got := HashBytes([]byte(input)); got != want {
			t.Errorf("HashBytes(%q) = %s, want %s", input, got, want)
		}
	}
}

func TestStreamingMatchesOneShot(t *testing.T) {
	h := NewHasher()
	h.Update([]byte("foo"))
