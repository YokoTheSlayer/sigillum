use std::collections::BTreeMap;
use std::fmt;

const MAX_DOCUMENT_BYTES: usize = 4 * 1024 * 1024;
const MAX_NESTING_DEPTH: usize = 128;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Value {
    Null,
    Bool(bool),
    Number,
    String(String),
    Array(Vec<Self>),
    Object(BTreeMap<String, Self>),
}

impl Value {
    pub(crate) fn as_object(&self) -> Option<&BTreeMap<String, Self>> {
        match self {
            Self::Object(value) => Some(value),
            _ => None,
        }
    }

    pub(crate) fn as_array(&self) -> Option<&[Self]> {
        match self {
            Self::Array(value) => Some(value),
            _ => None,
        }
    }

    pub(crate) fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }

    pub(crate) const fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(value) => Some(*value),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ParseError {
    offset: usize,
    message: &'static str,
}

impl fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} at byte {}", self.message, self.offset)
    }
}

pub(crate) fn parse(input: &[u8]) -> Result<Value, ParseError> {
    if input.len() > MAX_DOCUMENT_BYTES {
        return Err(ParseError {
            offset: 0,
            message: "JSON document exceeds size limit",
        });
    }

    let mut parser = Parser { input, offset: 0 };
    parser.skip_whitespace();
    let value = parser.parse_value(0)?;
    parser.skip_whitespace();
    if parser.offset == input.len() {
        Ok(value)
    } else {
        Err(parser.error("trailing data after JSON document"))
    }
}

struct Parser<'a> {
    input: &'a [u8],
    offset: usize,
}

impl Parser<'_> {
    fn parse_value(&mut self, depth: usize) -> Result<Value, ParseError> {
        if depth > MAX_NESTING_DEPTH {
            return Err(self.error("JSON nesting exceeds limit"));
        }
        match self.peek() {
            Some(b'n') => self.parse_literal(b"null", Value::Null),
            Some(b't') => self.parse_literal(b"true", Value::Bool(true)),
            Some(b'f') => self.parse_literal(b"false", Value::Bool(false)),
            Some(b'"') => self.parse_string().map(Value::String),
            Some(b'[') => self.parse_array(depth + 1),
            Some(b'{') => self.parse_object(depth + 1),
            Some(b'-' | b'0'..=b'9') => self.parse_number(),
            Some(_) => Err(self.error("unexpected JSON token")),
            None => Err(self.error("unexpected end of JSON document")),
        }
    }

    fn parse_literal(&mut self, expected: &[u8], value: Value) -> Result<Value, ParseError> {
        if self.remaining().starts_with(expected) {
            self.offset += expected.len();
            Ok(value)
        } else {
            Err(self.error("invalid JSON literal"))
        }
    }

    fn parse_array(&mut self, depth: usize) -> Result<Value, ParseError> {
        self.expect(b'[')?;
        self.skip_whitespace();
        let mut values = Vec::new();
        if self.consume(b']') {
            return Ok(Value::Array(values));
        }

        loop {
            self.skip_whitespace();
            values.push(self.parse_value(depth)?);
            self.skip_whitespace();
            if self.consume(b']') {
                return Ok(Value::Array(values));
            }
            self.expect(b',')?;
        }
    }

    fn parse_object(&mut self, depth: usize) -> Result<Value, ParseError> {
        self.expect(b'{')?;
        self.skip_whitespace();
        let mut values = BTreeMap::new();
        if self.consume(b'}') {
            return Ok(Value::Object(values));
        }

        loop {
            self.skip_whitespace();
            let key = self.parse_string()?;
            self.skip_whitespace();
            self.expect(b':')?;
            self.skip_whitespace();
            let value = self.parse_value(depth)?;
            if values.insert(key, value).is_some() {
                return Err(self.error("duplicate JSON object key"));
            }
            self.skip_whitespace();
            if self.consume(b'}') {
                return Ok(Value::Object(values));
            }
            self.expect(b',')?;
        }
    }

    fn parse_number(&mut self) -> Result<Value, ParseError> {
        let start = self.offset;
        self.consume(b'-');
        match self.peek() {
            Some(b'0') => self.offset += 1,
            Some(b'1'..=b'9') => {
                self.offset += 1;
                while matches!(self.peek(), Some(b'0'..=b'9')) {
                    self.offset += 1;
                }
            }
            _ => return Err(self.error("invalid JSON number")),
        }

        if self.consume(b'.') {
            let fraction_start = self.offset;
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.offset += 1;
            }
            if self.offset == fraction_start {
                return Err(self.error("invalid JSON number fraction"));
            }
        }

        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.offset += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.offset += 1;
            }
            let exponent_start = self.offset;
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.offset += 1;
            }
            if self.offset == exponent_start {
                return Err(self.error("invalid JSON number exponent"));
            }
        }

        if self.offset == start {
            Err(self.error("invalid JSON number"))
        } else {
            Ok(Value::Number)
        }
    }

    #[allow(clippy::too_many_lines)]
    fn parse_string(&mut self) -> Result<String, ParseError> {
        self.expect(b'"')?;
        let mut output = Vec::new();
        loop {
            let byte = self
                .next()
                .ok_or_else(|| self.error("unterminated JSON string"))?;
            match byte {
                b'"' => {
                    return String::from_utf8(output)
                        .map_err(|_| self.error("JSON string is not valid UTF-8"));
                }
                b'\\' => match self
                    .next()
                    .ok_or_else(|| self.error("unterminated JSON escape"))?
                {
                    b'"' => output.push(b'"'),
                    b'\\' => output.push(b'\\'),
                    b'/' => output.push(b'/'),
                    b'b' => output.push(0x08),
                    b'f' => output.push(0x0c),
                    b'n' => output.push(b'\n'),
                    b'r' => output.push(b'\r'),
                    b't' => output.push(b'\t'),
                    b'u' => {
                        let first = self.parse_hex_quad()?;
                        let code_point = if (0xd800..=0xdbff).contains(&first) {
                            self.expect(b'\\')?;
                            self.expect(b'u')?;
                            let second = self.parse_hex_quad()?;
                            if !(0xdc00..=0xdfff).contains(&second) {
                                return Err(self.error("invalid low Unicode surrogate"));
                            }
                            0x1_0000
                                + ((u32::from(first) - 0xd800) << 10)
                                + (u32::from(second) - 0xdc00)
                        } else if (0xdc00..=0xdfff).contains(&first) {
                            return Err(self.error("unexpected low Unicode surrogate"));
                        } else {
                            u32::from(first)
                        };
                        let character = char::from_u32(code_point)
                            .ok_or_else(|| self.error("invalid Unicode code point"))?;
                        let mut encoded = [0_u8; 4];
                        output.extend_from_slice(character.encode_utf8(&mut encoded).as_bytes());
                    }
                    _ => return Err(self.error("invalid JSON escape")),
                },
                0x00..=0x1f => return Err(self.error("control character in JSON string")),
                _ => output.push(byte),
            }
        }
    }

    fn parse_hex_quad(&mut self) -> Result<u16, ParseError> {
        let mut value = 0_u16;
        for _ in 0..4 {
            let digit = self
                .next()
                .and_then(hex_value)
                .ok_or_else(|| self.error("invalid Unicode escape"))?;
            value = (value << 4) | u16::from(digit);
        }
        Ok(value)
    }

    fn expect(&mut self, expected: u8) -> Result<(), ParseError> {
        if self.consume(expected) {
            Ok(())
        } else {
            Err(self.error("unexpected JSON delimiter"))
        }
    }

    fn consume(&mut self, expected: u8) -> bool {
        if self.peek() == Some(expected) {
            self.offset += 1;
            true
        } else {
            false
        }
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.offset += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.offset).copied()
    }

    fn next(&mut self) -> Option<u8> {
        let value = self.peek()?;
        self.offset += 1;
        Some(value)
    }

    fn remaining(&self) -> &[u8] {
        &self.input[self.offset..]
    }

    const fn error(&self, message: &'static str) -> ParseError {
        ParseError {
            offset: self.offset,
            message,
        }
    }
}

const fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{parse, Value};

    #[test]
    fn parses_nested_protocol_values_and_unicode() {
        let value = parse(br#"{"root":{"path":"/tmp/\u03bb"},"ready":true,"files":["a",null,1]}"#)
            .expect("valid JSON");
        let object = value.as_object().expect("root object");

        assert_eq!(object.get("ready").and_then(Value::as_bool), Some(true));
        assert_eq!(
            object
                .get("root")
                .and_then(Value::as_object)
                .and_then(|root| root.get("path"))
                .and_then(Value::as_str),
            Some("/tmp/λ")
        );
    }

    #[test]
    fn rejects_duplicate_keys_and_trailing_data() {
        assert!(parse(br#"{"a":1,"a":2}"#).is_err());
        assert!(parse(br"{} true").is_err());
    }

    #[test]
    fn rejects_invalid_surrogates() {
        assert!(parse(br#""\ud800x""#).is_err());
        assert!(parse(br#""\udc00""#).is_err());
    }
}
