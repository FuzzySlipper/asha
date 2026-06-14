//! A tiny self-contained JSON value parser and a deterministic JSON writer.
//!
//! The workspace hand-rolls small JSON readers per crate (the policy is no serde in
//! the engine); this mirrors `core-scene::json`'s approach. The reader parses the
//! object/array/string/number/bool/null subset the source-mesh format uses; the
//! writer emits stable, two-space-indented, LF-terminated output so generated
//! artifacts are byte-reproducible and diffable.

// ── Reader ────────────────────────────────────────────────────────────────────

/// A parsed JSON value (the subset the importer needs).
#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    /// Parse a complete JSON document, rejecting trailing input.
    pub fn parse(input: &str) -> Result<Json, String> {
        let chars: Vec<char> = input.chars().collect();
        let mut p = Parser { chars, pos: 0 };
        p.skip_ws();
        let v = p.value()?;
        p.skip_ws();
        if p.pos != p.chars.len() {
            return Err(format!("trailing input at position {}", p.pos));
        }
        Ok(v)
    }

    /// Look up a key on an object value.
    pub fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Obj(entries) => entries.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    /// The object's keys in document order, or empty for a non-object.
    pub fn keys(&self) -> Vec<&str> {
        match self {
            Json::Obj(entries) => entries.iter().map(|(k, _)| k.as_str()).collect(),
            _ => Vec::new(),
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Json::Num(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Json::Num(n) if n.fract() == 0.0 && *n >= 0.0 => Some(*n as u64),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[Json]> {
        match self {
            Json::Arr(items) => Some(items),
            _ => None,
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Json::Null)
    }
}

struct Parser {
    chars: Vec<char>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(' ' | '\t' | '\n' | '\r')) {
            self.pos += 1;
        }
    }

    fn value(&mut self) -> Result<Json, String> {
        self.skip_ws();
        match self.peek() {
            Some('{') => self.object(),
            Some('[') => self.array(),
            Some('"') => Ok(Json::Str(self.string()?)),
            Some('t') | Some('f') => self.boolean(),
            Some('n') => self.null(),
            Some(c) if c == '-' || c.is_ascii_digit() => self.number(),
            other => Err(format!("unexpected {other:?} at {}", self.pos)),
        }
    }

    fn object(&mut self) -> Result<Json, String> {
        self.expect('{')?;
        let mut entries = Vec::new();
        self.skip_ws();
        if self.peek() == Some('}') {
            self.pos += 1;
            return Ok(Json::Obj(entries));
        }
        loop {
            self.skip_ws();
            let key = self.string()?;
            self.skip_ws();
            self.expect(':')?;
            let val = self.value()?;
            entries.push((key, val));
            self.skip_ws();
            match self.bump() {
                Some(',') => continue,
                Some('}') => break,
                other => return Err(format!("expected ',' or '}}', got {other:?}")),
            }
        }
        Ok(Json::Obj(entries))
    }

    fn array(&mut self) -> Result<Json, String> {
        self.expect('[')?;
        let mut items = Vec::new();
        self.skip_ws();
        if self.peek() == Some(']') {
            self.pos += 1;
            return Ok(Json::Arr(items));
        }
        loop {
            items.push(self.value()?);
            self.skip_ws();
            match self.bump() {
                Some(',') => continue,
                Some(']') => break,
                other => return Err(format!("expected ',' or ']', got {other:?}")),
            }
        }
        Ok(Json::Arr(items))
    }

    fn string(&mut self) -> Result<String, String> {
        self.expect('"')?;
        let mut out = String::new();
        loop {
            match self.bump() {
                Some('"') => break,
                Some('\\') => match self.bump() {
                    Some('"') => out.push('"'),
                    Some('\\') => out.push('\\'),
                    Some('/') => out.push('/'),
                    Some('n') => out.push('\n'),
                    Some('t') => out.push('\t'),
                    Some('r') => out.push('\r'),
                    other => return Err(format!("bad escape {other:?}")),
                },
                Some(c) => out.push(c),
                None => return Err("unterminated string".into()),
            }
        }
        Ok(out)
    }

    fn boolean(&mut self) -> Result<Json, String> {
        if self.consume("true") {
            Ok(Json::Bool(true))
        } else if self.consume("false") {
            Ok(Json::Bool(false))
        } else {
            Err(format!("bad literal at {}", self.pos))
        }
    }

    fn null(&mut self) -> Result<Json, String> {
        if self.consume("null") {
            Ok(Json::Null)
        } else {
            Err(format!("bad literal at {}", self.pos))
        }
    }

    fn number(&mut self) -> Result<Json, String> {
        let start = self.pos;
        if self.peek() == Some('-') {
            self.pos += 1;
        }
        while matches!(self.peek(), Some(c) if c.is_ascii_digit() || c == '.' || c == 'e' || c == 'E' || c == '+' || c == '-')
        {
            self.pos += 1;
        }
        let s: String = self.chars[start..self.pos].iter().collect();
        s.parse::<f64>()
            .map(Json::Num)
            .map_err(|_| format!("bad number `{s}`"))
    }

    fn expect(&mut self, c: char) -> Result<(), String> {
        if self.bump() == Some(c) {
            Ok(())
        } else {
            Err(format!("expected '{c}' at {}", self.pos))
        }
    }

    fn consume(&mut self, lit: &str) -> bool {
        let end = self.pos + lit.len();
        if end <= self.chars.len() && self.chars[self.pos..end].iter().collect::<String>() == lit {
            self.pos = end;
            true
        } else {
            false
        }
    }
}

// ── Writer ────────────────────────────────────────────────────────────────────

/// A deterministic JSON writer building stable, indented, LF-terminated output.
/// Object keys are emitted in insertion order — callers insert in a fixed order so
/// the bytes are reproducible and diffable.
#[derive(Debug, Default)]
pub struct JsonWriter {
    out: String,
    depth: usize,
}

impl JsonWriter {
    pub fn new() -> Self {
        JsonWriter::default()
    }

    /// Finish, returning the document with a single trailing newline.
    pub fn finish(mut self) -> String {
        if !self.out.ends_with('\n') {
            self.out.push('\n');
        }
        self.out
    }

    fn indent(&mut self) {
        for _ in 0..self.depth {
            self.out.push_str("  ");
        }
    }

    /// Render a number with a stable form: integers without a decimal point,
    /// finite floats via Rust's shortest round-trip formatting.
    pub fn number(n: f64) -> String {
        if n.fract() == 0.0 && n.is_finite() && n.abs() < 1e15 {
            format!("{}", n as i64)
        } else {
            format!("{n}")
        }
    }

    fn escape(s: &str) -> String {
        let mut out = String::with_capacity(s.len() + 2);
        out.push('"');
        for c in s.chars() {
            match c {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\t' => out.push_str("\\t"),
                '\r' => out.push_str("\\r"),
                _ => out.push(c),
            }
        }
        out.push('"');
        out
    }

    /// Begin an object on a fresh line at the current indent.
    pub fn begin_object(&mut self) {
        self.out.push_str("{\n");
        self.depth += 1;
    }

    /// End an object. `last` marks whether a trailing comma is omitted.
    pub fn end_object(&mut self, trailing_comma: bool) {
        self.depth -= 1;
        self.indent();
        self.out.push('}');
        if trailing_comma {
            self.out.push(',');
        }
        self.out.push('\n');
    }

    /// Write a string field.
    pub fn field_str(&mut self, key: &str, value: &str, last: bool) {
        self.indent();
        self.out
            .push_str(&format!("{}: {}", Self::escape(key), Self::escape(value)));
        self.comma(last);
    }

    /// Write a nullable string field (`null` when absent).
    pub fn field_opt_str(&mut self, key: &str, value: Option<&str>, last: bool) {
        self.indent();
        match value {
            Some(v) => self
                .out
                .push_str(&format!("{}: {}", Self::escape(key), Self::escape(v))),
            None => self.out.push_str(&format!("{}: null", Self::escape(key))),
        }
        self.comma(last);
    }

    /// Render an `f32` with shortest round-trip form (integers without a point).
    pub fn number_f32(n: f32) -> String {
        if n.fract() == 0.0 && n.is_finite() && n.abs() < 1e15 {
            format!("{}", n as i64)
        } else {
            format!("{n}")
        }
    }

    /// Write an `f32` number field (avoids f64-widening noise in the output).
    pub fn field_f32(&mut self, key: &str, value: f32, last: bool) {
        self.indent();
        self.out.push_str(&format!(
            "{}: {}",
            Self::escape(key),
            Self::number_f32(value)
        ));
        self.comma(last);
    }

    /// Write an `f32`-array field on one line.
    pub fn field_f32_array(&mut self, key: &str, values: &[f32], last: bool) {
        self.indent();
        let inner: Vec<String> = values.iter().map(|v| Self::number_f32(*v)).collect();
        self.out
            .push_str(&format!("{}: [{}]", Self::escape(key), inner.join(", ")));
        self.comma(last);
    }

    /// Write a number field.
    pub fn field_num(&mut self, key: &str, value: f64, last: bool) {
        self.indent();
        self.out
            .push_str(&format!("{}: {}", Self::escape(key), Self::number(value)));
        self.comma(last);
    }

    /// Write a boolean field.
    pub fn field_bool(&mut self, key: &str, value: bool, last: bool) {
        self.indent();
        self.out
            .push_str(&format!("{}: {}", Self::escape(key), value));
        self.comma(last);
    }

    /// Write a number-array field on one line (compact, for vertex streams).
    pub fn field_num_array(&mut self, key: &str, values: &[f64], last: bool) {
        self.indent();
        let inner: Vec<String> = values.iter().map(|v| Self::number(*v)).collect();
        self.out
            .push_str(&format!("{}: [{}]", Self::escape(key), inner.join(", ")));
        self.comma(last);
    }

    /// Write a string-array field on one line.
    pub fn field_str_array(&mut self, key: &str, values: &[String], last: bool) {
        self.indent();
        let inner: Vec<String> = values.iter().map(|v| Self::escape(v)).collect();
        self.out
            .push_str(&format!("{}: [{}]", Self::escape(key), inner.join(", ")));
        self.comma(last);
    }

    /// Begin a nested-object field (`"key": {` … ), then write fields and call
    /// [`Self::end_object`].
    pub fn indent_field_object(&mut self, key: &str) {
        self.indent();
        self.out.push_str(&format!("{}: {{\n", Self::escape(key)));
        self.depth += 1;
    }

    /// Begin an array field; the caller emits elements then calls [`Self::end_array`].
    pub fn begin_array_field(&mut self, key: &str) {
        self.indent();
        self.out.push_str(&format!("{}: [\n", Self::escape(key)));
        self.depth += 1;
    }

    /// Indent for an array element that is an object (call before `begin_object`).
    pub fn array_element_indent(&mut self) {
        self.indent();
    }

    pub fn end_array(&mut self, last: bool) {
        self.depth -= 1;
        self.indent();
        self.out.push(']');
        self.comma(last);
    }

    fn comma(&mut self, last: bool) {
        if !last {
            self.out.push(',');
        }
        self.out.push('\n');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_a_small_object() {
        let v = Json::parse(r#"{ "a": 1, "b": [true, null, "x"], "c": -2.5 }"#).unwrap();
        assert_eq!(v.get("a").and_then(Json::as_u64), Some(1));
        assert_eq!(v.get("c").and_then(Json::as_f64), Some(-2.5));
        let arr = v.get("b").and_then(Json::as_array).unwrap();
        assert_eq!(arr.len(), 3);
        assert!(arr[1].is_null());
    }

    #[test]
    fn rejects_trailing_input() {
        assert!(Json::parse("{} junk").is_err());
    }

    #[test]
    fn writer_is_stable_and_lf_terminated() {
        let mut w = JsonWriter::new();
        w.begin_object();
        w.field_str("name", "x", false);
        w.field_num("v", 2.0, true);
        w.end_object(false);
        let s = w.finish();
        assert_eq!(s, "{\n  \"name\": \"x\",\n  \"v\": 2\n}\n");
    }
}
