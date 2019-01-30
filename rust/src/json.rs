//! A tiny, dependency-free JSON writer and reader.
//!
//! Cairn manifests are simple, flat-ish JSON documents. Rather than pull in a
//! serialization crate, we ship a focused implementation that supports exactly
//! the value shapes the manifest format uses: objects, arrays, strings,
//! unsigned integers, and booleans. The writer emits **canonical** JSON with
//! keys in insertion order and no insignificant whitespace, so that manifest
//! bytes are stable and reproducible across runs.

use std::collections::BTreeMap;
use std::fmt::Write as _;

/// A JSON value restricted to the shapes Cairn needs.
#[derive(Clone, Debug, PartialEq)]
pub enum Json {
    Str(String),
    Uint(u64),
    Bool(bool),
    Array(Vec<Json>),
    /// Ordered map so serialization is deterministic.
    Object(Vec<(String, Json)>),
}

impl Json {
    /// Serialize to canonical, compact JSON (no extra whitespace).
    pub fn encode(&self) -> String {
        let mut out = String::new();
        self.write_to(&mut out);
        out
    }

    fn write_to(&self, out: &mut String) {
        match self {
            Json::Str(s) => write_json_string(s, out),
            Json::Uint(n) => {
                let _ = write!(out, "{n}");
            }
            Json::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            Json::Array(items) => {
                out.push('[');
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    item.write_to(out);
                }
                out.push(']');
            }
            Json::Object(entries) => {
                out.push('{');
                for (i, (k, v)) in entries.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    write_json_string(k, out);
                    out.push(':');
                    v.write_to(out);
                }
                out.push('}');
            }
        }
    }

    /// Borrow this value as an object's entries, if it is an object.
    pub fn as_object(&self) -> Option<&[(String, Json)]> {
        match self {
            Json::Object(e) => Some(e),
            _ => None,
        }
    }

    /// Look up a key in an object value.
    pub fn get(&self, key: &str) -> Option<&Json> {
        self.as_object()?
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
    }

    /// Borrow this value as a string.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }

    /// Borrow this value as an unsigned integer.
    pub fn as_uint(&self) -> Option<u64> {
        match self {
            Json::Uint(n) => Some(*n),
            _ => None,
        }
    }

    /// Borrow this value as a boolean.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Json::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Borrow this value as an array.
    pub fn as_array(&self) -> Option<&[Json]> {
        match self {
            Json::Array(a) => Some(a),
            _ => None,
        }
    }
}

/// Write a JSON string literal, escaping per RFC 8259.
fn write_json_string(s: &str, out: &mut String) {
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

/// Parse error with a byte offset for context.
#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub offset: usize,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "JSON parse error at byte {}: {}",
            self.offset, self.message
        )
    }
}

impl std::error::Error for ParseError {}

/// Parse a JSON document into a [`Json`] value.
pub fn parse(input: &str) -> Result<Json, ParseError> {
    let mut p = Parser {
        bytes: input.as_bytes(),
        pos: 0,
    };
    p.skip_ws();
    let value = p.parse_value()?;
    p.skip_ws();
    if p.pos != p.bytes.len() {
        return Err(p.err("trailing characters after JSON value"));
    }
    Ok(value)
}

struct Parser<'a> {
