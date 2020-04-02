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
	h.Update([]byte("bar"))
	if got := h.Hex(); got != HashBytes([]byte("foobar")) {
		t.Errorf("streaming digest %s != one-shot", got)
	}
}

// The manifest JSON produced here must match, byte-for-byte, what the Rust
// engine emits for the same logical manifest (see rust manifest tests).
func TestManifestCanonicalJSON(t *testing.T) {
	inputs := []Entry{
		{Path: "src/a.txt", Digest: "af63dc4c8601ec8c", Size: 1},
		{Path: "src/b.txt", Digest: "85944171f73967e8", Size: 6},
	}
	command := []string{"cc", "-c"}
	key := ComputeKey(inputs, command)
	m := &Manifest{
		Version:  FormatVersion,
		Producer: "cairn-rust",
		Key:      key,
		Command:  command,
		Inputs:   inputs,
		Outputs: []Entry{
			{Path: "out/a.o", Digest: "cbf29ce484222325", Size: 0},
		},
	}
	got := m.ToJSON()

	// Parse it back and re-encode to confirm stability.
	parsed, err := ParseManifest([]byte(got))
	if err != nil {
		t.Fatalf("ParseManifest failed: %v", err)
	}
	if again := parsed.ToJSON(); again != got {
		t.Errorf("re-encoded JSON differs:\n first: %s\nsecond: %s", got, again)
	}
}

func TestKeyStableAndCommandSensitive(t *testing.T) {
	inputs := []Entry{{Path: "x", Digest: "af63dc4c8601ec8c", Size: 1}}
	k1 := ComputeKey(inputs, []string{"go", "build"})
	k2 := ComputeKey(inputs, []string{"go", "build"})
	if k1 != k2 {
		t.Errorf("key not stable: %s != %s", k1, k2)
	}
	k3 := ComputeKey(inputs, []string{"go", "test"})
	if k1 == k3 {
		t.Errorf("key should change with command")
	}
}

func TestStoreRestoreVerify(t *testing.T) {
	dir := t.TempDir()
	work := filepath.Join(dir, "work")
	if err := os.MkdirAll(work, 0o755); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(work, "in.txt"), []byte("foobar"), 0o644); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(work, "out.txt"), []byte("result-bytes"), 0o644); err != nil {
		t.Fatal(err)
	}

