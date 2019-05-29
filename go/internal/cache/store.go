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
