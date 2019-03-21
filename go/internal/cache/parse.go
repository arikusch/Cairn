package cache

import (
	"bytes"
	"encoding/json"
	"fmt"
)

// wireManifest mirrors the JSON structure for decoding. We decode with the
// standard library, then convert to the strongly typed Manifest. Encoding is
