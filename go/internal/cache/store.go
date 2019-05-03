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
