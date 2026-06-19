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
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn err(&self, msg: &str) -> ParseError {
        ParseError {
            message: msg.to_string(),
            offset: self.pos,
        }
    }

    fn skip_ws(&mut self) {
        while self.pos < self.bytes.len() {
            match self.bytes[self.pos] {
                b' ' | b'\t' | b'\n' | b'\r' => self.pos += 1,
                _ => break,
            }
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn parse_value(&mut self) -> Result<Json, ParseError> {
        self.skip_ws();
        match self.peek() {
            Some(b'"') => self.parse_string().map(Json::Str),
            Some(b'{') => self.parse_object(),
            Some(b'[') => self.parse_array(),
            Some(b't') | Some(b'f') => self.parse_bool(),
            Some(c) if c.is_ascii_digit() => self.parse_uint(),
            _ => Err(self.err("expected a JSON value")),
        }
    }

    fn parse_bool(&mut self) -> Result<Json, ParseError> {
        if self.bytes[self.pos..].starts_with(b"true") {
            self.pos += 4;
            Ok(Json::Bool(true))
        } else if self.bytes[self.pos..].starts_with(b"false") {
            self.pos += 5;
            Ok(Json::Bool(false))
        } else {
            Err(self.err("invalid literal"))
        }
    }

    fn parse_uint(&mut self) -> Result<Json, ParseError> {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                self.pos += 1;
            } else {
                break;
            }
        }
        let slice = &self.bytes[start..self.pos];
        let text = std::str::from_utf8(slice).map_err(|_| self.err("invalid number"))?;
        let n = text
            .parse::<u64>()
            .map_err(|_| self.err("number out of range"))?;
        Ok(Json::Uint(n))
    }

    fn parse_string(&mut self) -> Result<String, ParseError> {
        // Consume opening quote.
        self.pos += 1;
        let mut s = String::new();
        loop {
            match self.peek() {
                None => return Err(self.err("unterminated string")),
                Some(b'"') => {
                    self.pos += 1;
                    return Ok(s);
                }
                Some(b'\\') => {
                    self.pos += 1;
                    match self.peek() {
                        Some(b'"') => s.push('"'),
                        Some(b'\\') => s.push('\\'),
                        Some(b'/') => s.push('/'),
                        Some(b'n') => s.push('\n'),
                        Some(b'r') => s.push('\r'),
                        Some(b't') => s.push('\t'),
                        Some(b'b') => s.push('\u{0008}'),
                        Some(b'f') => s.push('\u{000c}'),
                        Some(b'u') => {
                            let cp = self.parse_unicode_escape()?;
                            s.push(cp);
                            // parse_unicode_escape leaves pos on last hex digit.
                            continue;
                        }
                        _ => return Err(self.err("invalid escape sequence")),
                    }
                    self.pos += 1;
                }
                Some(_) => {
                    // Copy one UTF-8 encoded char.
                    let rest = &self.bytes[self.pos..];
                    let ch_len = utf8_len(rest[0]);
                    if self.pos + ch_len > self.bytes.len() {
                        return Err(self.err("truncated UTF-8 sequence"));
                    }
                    let chunk = &rest[..ch_len];
                    let text = std::str::from_utf8(chunk).map_err(|_| self.err("invalid UTF-8"))?;
                    s.push_str(text);
                    self.pos += ch_len;
                }
            }
        }
    }

    fn parse_unicode_escape(&mut self) -> Result<char, ParseError> {
        // pos is on 'u'; four hex digits follow.
        if self.pos + 4 >= self.bytes.len() {
            return Err(self.err("truncated \\u escape"));
        }
        let hex = &self.bytes[self.pos + 1..self.pos + 5];
        let text = std::str::from_utf8(hex).map_err(|_| self.err("invalid \\u escape"))?;
        let cp = u32::from_str_radix(text, 16).map_err(|_| self.err("invalid \\u escape"))?;
        self.pos += 4; // move to last hex digit; caller advances one more.
        char::from_u32(cp).ok_or_else(|| self.err("invalid code point"))
    }

    fn parse_array(&mut self) -> Result<Json, ParseError> {
        self.pos += 1; // consume '['
        let mut items = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(Json::Array(items));
        }
        loop {
            let value = self.parse_value()?;
            items.push(value);
            self.skip_ws();
            match self.peek() {
                Some(b',') => {
                    self.pos += 1;
                }
                Some(b']') => {
                    self.pos += 1;
                    return Ok(Json::Array(items));
                }
                _ => return Err(self.err("expected ',' or ']' in array")),
            }
        }
    }

    fn parse_object(&mut self) -> Result<Json, ParseError> {
        self.pos += 1; // consume '{'
        let mut entries: Vec<(String, Json)> = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(Json::Object(entries));
        }
        loop {
            self.skip_ws();
            if self.peek() != Some(b'"') {
                return Err(self.err("expected string key in object"));
            }
            let key = self.parse_string()?;
            self.skip_ws();
            if self.peek() != Some(b':') {
                return Err(self.err("expected ':' after object key"));
            }
            self.pos += 1;
            let value = self.parse_value()?;
            entries.push((key, value));
            self.skip_ws();
            match self.peek() {
                Some(b',') => {
                    self.pos += 1;
                }
                Some(b'}') => {
                    self.pos += 1;
                    return Ok(Json::Object(entries));
                }
                _ => return Err(self.err("expected ',' or '}' in object")),
            }
        }
    }
}

/// Length in bytes of a UTF-8 sequence given its leading byte.
fn utf8_len(lead: u8) -> usize {
    if lead < 0x80 {
        1
    } else if lead >> 5 == 0b110 {
        2
    } else if lead >> 4 == 0b1110 {
        3
    } else {
        4
    }
}

/// Helper to build an object from owned pairs while preserving order.
pub fn object(pairs: Vec<(&str, Json)>) -> Json {
    Json::Object(pairs.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
}

/// Sort a set of (key, value) string pairs deterministically for hashing.
pub fn sorted_map(map: BTreeMap<String, String>) -> Json {
    Json::Object(map.into_iter().map(|(k, v)| (k, Json::Str(v))).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_object() {
        let v = object(vec![
            ("name", Json::Str("cairn".into())),
            ("count", Json::Uint(3)),
            ("ok", Json::Bool(true)),
            (
                "items",
                Json::Array(vec![Json::Str("a".into()), Json::Str("b".into())]),
            ),
        ]);
        let text = v.encode();
        assert_eq!(
            text,
            r#"{"name":"cairn","count":3,"ok":true,"items":["a","b"]}"#
        );
        let parsed = parse(&text).unwrap();
        assert_eq!(parsed, v);
    }

    #[test]
    fn escapes() {
        let v = Json::Str("line1\nline2\t\"quote\"".into());
        let text = v.encode();
        let parsed = parse(&text).unwrap();
        assert_eq!(parsed, v);
    }

    #[test]
    fn rejects_trailing() {
        assert!(parse("{}garbage").is_err());
    }
}
