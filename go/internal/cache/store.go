package cache

import (
	"fmt"
	"os"
	"path/filepath"
)

// Store is the on-disk content-addressed store, byte-compatible with the Rust
// engine's layout:
//
//	<root>/objects/<aa>/<digest>
//	<root>/manifests/<key>.json
type Store struct {
	root string
}

// OpenStore opens (creating if necessary) a store rooted at root.
func OpenStore(root string) (*Store, error) {
	if err := os.MkdirAll(filepath.Join(root, "objects"), 0o755); err != nil {
		return nil, err
	}
	if err := os.MkdirAll(filepath.Join(root, "manifests"), 0o755); err != nil {
		return nil, err
	}
	return &Store{root: root}, nil
}

func (s *Store) objectPath(digest string) string {
	shard := digest
	if len(digest) >= 2 {
		shard = digest[:2]
	}
	return filepath.Join(s.root, "objects", shard, digest)
}

func (s *Store) manifestPath(key string) string {
	return filepath.Join(s.root, "manifests", key+".json")
}

// PutBlob stores content by its digest, returning the digest. Idempotent.
func (s *Store) PutBlob(content []byte) (string, error) {
	digest := HashBytes(content)
	dest := s.objectPath(digest)
	if _, err := os.Stat(dest); err == nil {
		return digest, nil
	}
	if err := os.MkdirAll(filepath.Dir(dest), 0o755); err != nil {
		return "", err
	}
	if err := atomicWrite(dest, content); err != nil {
		return "", err
	}
	return digest, nil
}

// PutFile stores a file's contents as a blob, returning digest and size.
func (s *Store) PutFile(path string) (string, uint64, error) {
	content, err := os.ReadFile(path)
	if err != nil {
		return "", 0, err
	}
	digest, err := s.PutBlob(content)
	if err != nil {
		return "", 0, err
	}
	return digest, uint64(len(content)), nil
}

// GetBlob reads a blob by digest.
func (s *Store) GetBlob(digest string) ([]byte, error) {
	return os.ReadFile(s.objectPath(digest))
}

// HasBlob reports whether a blob with the digest exists.
func (s *Store) HasBlob(digest string) bool {
	_, err := os.Stat(s.objectPath(digest))
	return err == nil
}

// PutManifest persists a manifest under its key using canonical JSON.
func (s *Store) PutManifest(m *Manifest) error {
	dest := s.manifestPath(m.Key)
	if err := os.MkdirAll(filepath.Dir(dest), 0o755); err != nil {
		return err
	}
	return atomicWrite(dest, []byte(m.ToJSON()))
}

// GetManifest loads a manifest by key, returning (nil, nil) on a cache miss.
func (s *Store) GetManifest(key string) (*Manifest, error) {
	path := s.manifestPath(key)
	data, err := os.ReadFile(path)
	if os.IsNotExist(err) {
		return nil, nil
	}
	if err != nil {
		return nil, err
	}
	return ParseManifest(data)
}

// HasManifest reports whether a manifest exists for the key.
func (s *Store) HasManifest(key string) bool {
	_, err := os.Stat(s.manifestPath(key))
	return err == nil
}

// HashInputs hashes input files relative to base into sorted entries.
func HashInputs(base string, paths []string) ([]Entry, error) {
	entries := make([]Entry, 0, len(paths))
	for _, p := range paths {
		digest, size, err := HashFile(filepath.Join(base, p))
		if err != nil {
			return nil, fmt.Errorf("hashing input %q: %w", p, err)
		}
		entries = append(entries, Entry{Path: NormalizePath(p), Digest: digest, Size: size})
	}
	SortEntries(entries)
	return entries, nil
}

// StoreOutputs stores output files as blobs and returns sorted entries.
func (s *Store) StoreOutputs(base string, paths []string) ([]Entry, error) {
	entries := make([]Entry, 0, len(paths))
	for _, p := range paths {
		digest, size, err := s.PutFile(filepath.Join(base, p))
		if err != nil {
			return nil, fmt.Errorf("storing output %q: %w", p, err)
		}
		entries = append(entries, Entry{Path: NormalizePath(p), Digest: digest, Size: size})
	}
	SortEntries(entries)
	return entries, nil
}

// RestoreOutputs writes all outputs from a manifest into base, returning the
// count. Fails if any referenced blob is missing.
func (s *Store) RestoreOutputs(base string, m *Manifest) (int, error) {
	for _, o := range m.Outputs {
		if !s.HasBlob(o.Digest) {
			return 0, fmt.Errorf("missing blob %s for output %s", o.Digest, o.Path)
		}
	}
	written := 0
	for _, o := range m.Outputs {
		content, err := s.GetBlob(o.Digest)
		if err != nil {
			return written, err
		}
		dest := filepath.Join(base, filepath.FromSlash(o.Path))
		if err := os.MkdirAll(filepath.Dir(dest), 0o755); err != nil {
			return written, err
		}
		if err := os.WriteFile(dest, content, 0o644); err != nil {
			return written, err
		}
