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
