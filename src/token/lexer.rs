use super::TokenPosition;
use crate::config::CloneConfig;
use crate::error::{CloneError, Result};
use crate::model::Language;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    Identifier,
    Number,
    String,
    Syntax,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct LexedToken {
    pub position: TokenPosition,
    pub(super) kind: Kind,
}

pub(crate) fn lex(
    source: &str,
    language: Language,
    config: CloneConfig,
) -> Result<Vec<LexedToken>> {
    let mut tokens = Vec::with_capacity(source.len() / 4);
    let bytes = source.as_bytes();
    let mut cursor = 0;
    let mut line = 1_u32;
    while cursor < bytes.len() {
        if bytes[cursor].is_ascii_whitespace() {
            while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
                line = line.saturating_add(u32::from(bytes[cursor] == b'\n'));
                cursor += 1;
            }
            continue;
        }
        if let Some(end) = skip_comment(source, cursor, language) {
            line = line.saturating_add(newlines(&source[cursor..end]));
            cursor = end;
            continue;
        }
        let (end, kind) = token_end(source, cursor, language);
        let token_lines = newlines(&source[cursor..end]);
        tokens.push(LexedToken {
            position: TokenPosition {
                start_byte: cursor,
                end_byte: end,
                start_line: line,
                end_line: line.saturating_add(token_lines),
            },
            kind,
        });
        if tokens.len() > config.max_tokens_per_fragment {
            return Err(CloneError::CapacityExceeded {
                resource: "tokens per fragment",
                limit: config.max_tokens_per_fragment,
            });
        }
        line = line.saturating_add(token_lines);
        cursor = end;
    }
    Ok(tokens)
}

fn newlines(text: &str) -> u32 {
    if !text.as_bytes().contains(&b'\n') {
        return 0;
    }
    u32::try_from(text.bytes().filter(|byte| *byte == b'\n').count()).unwrap_or(u32::MAX)
}

fn skip_comment(source: &str, cursor: usize, language: Language) -> Option<usize> {
    let bytes = source.as_bytes();
    let byte = bytes[cursor];
    if matches!(language, Language::Python | Language::Bash) && byte == b'#' {
        return Some(line_end(source, cursor + 1));
    }
    if matches!(
        language,
        Language::Rust
            | Language::Go
            | Language::C
            | Language::Cpp
            | Language::JavaScript
            | Language::TypeScript
            | Language::Java
            | Language::CSharp
    ) && byte == b'/'
    {
        match bytes.get(cursor + 1) {
            Some(b'/') => return Some(line_end(source, cursor + 2)),
            Some(b'*') => {
                return Some(block_comment_end(
                    source,
                    cursor + 2,
                    language == Language::Rust,
                ));
            }
            _ => {}
        }
    }
    if language == Language::Sql && byte == b'-' && bytes.get(cursor + 1) == Some(&b'-') {
        return Some(line_end(source, cursor + 2));
    }
    None
}

fn line_end(source: &str, cursor: usize) -> usize {
    source[cursor..]
        .find('\n')
        .map_or(source.len(), |offset| cursor + offset + 1)
}

fn block_comment_end(source: &str, mut cursor: usize, nested: bool) -> usize {
    let bytes = source.as_bytes();
    let mut depth = 1_usize;
    while cursor + 1 < bytes.len() {
        if nested && &bytes[cursor..cursor + 2] == b"/*" {
            depth += 1;
            cursor += 2;
        } else if &bytes[cursor..cursor + 2] == b"*/" {
            depth -= 1;
            cursor += 2;
            if depth == 0 {
                return cursor;
            }
        } else {
            cursor += if bytes[cursor].is_ascii() {
                1
            } else {
                char_len(source, cursor)
            };
        }
    }
    source.len()
}

fn token_end(source: &str, cursor: usize, language: Language) -> (usize, Kind) {
    let bytes = source.as_bytes();
    let byte = bytes[cursor];
    if is_identifier_start(byte)
        || byte >= 0x80
            && source[cursor..]
                .chars()
                .next()
                .is_some_and(char::is_alphanumeric)
    {
        return (identifier_end(source, cursor), Kind::Identifier);
    }
    if byte.is_ascii_digit() {
        return (number_end(source, cursor), Kind::Number);
    }
    if matches!(byte, b'\'' | b'"' | b'`') {
        return (
            string_end(source, cursor, byte, language == Language::Python),
            Kind::String,
        );
    }
    (operator_end(source, cursor), Kind::Syntax)
}

fn identifier_end(source: &str, mut cursor: usize) -> usize {
    let bytes = source.as_bytes();
    while cursor < bytes.len() {
        let byte = bytes[cursor];
        if is_identifier_continue(byte) {
            cursor += 1;
        } else if byte >= 0x80 {
            let Some(character) = source[cursor..].chars().next() else {
                break;
            };
            if !character.is_alphanumeric() {
                break;
            }
            cursor += character.len_utf8();
        } else {
            break;
        }
    }
    cursor
}

fn number_end(source: &str, mut cursor: usize) -> usize {
    let bytes = source.as_bytes();
    let mut exponent = false;
    while let Some(&byte) = bytes.get(cursor) {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.') {
            exponent = matches!(byte, b'e' | b'E' | b'p' | b'P');
            cursor += 1;
        } else if exponent && matches!(byte, b'+' | b'-') {
            exponent = false;
            cursor += 1;
        } else {
            break;
        }
    }
    cursor
}

fn string_end(source: &str, cursor: usize, quote: u8, python: bool) -> usize {
    let bytes = source.as_bytes();
    let triple = python
        && cursor + 2 < bytes.len()
        && bytes[cursor + 1] == quote
        && bytes[cursor + 2] == quote;
    let mut current = cursor + if triple { 3 } else { 1 };
    while current < bytes.len() {
        if bytes[current] == b'\\' {
            current = (current + 2).min(bytes.len());
            continue;
        }
        if triple {
            if current + 2 < bytes.len()
                && bytes[current] == quote
                && bytes[current + 1] == quote
                && bytes[current + 2] == quote
            {
                return current + 3;
            }
        } else if bytes[current] == quote {
            return current + 1;
        }
        current += if bytes[current].is_ascii() {
            1
        } else {
            char_len(source, current)
        };
    }
    bytes.len()
}

fn operator_end(source: &str, cursor: usize) -> usize {
    let bytes = source.as_bytes();
    let rest = &bytes[cursor..];
    let length = match bytes[cursor] {
        b'>' if rest.starts_with(b">>>=") => 4,
        b'>' if rest.starts_with(b">>=") => 3,
        b'<' if rest.starts_with(b"<<=") => 3,
        b'=' if rest.starts_with(b"===") => 3,
        b'!' if rest.starts_with(b"!==") => 3,
        b'.' if rest.starts_with(b"...") => 3,
        b'>' if starts_any(rest, &[b">=", b">>"]) => 2,
        b'<' if starts_any(rest, &[b"<=", b"<<", b"<-"]) => 2,
        b'=' if starts_any(rest, &[b"=>", b"=="]) => 2,
        b'!' if rest.starts_with(b"!=") => 2,
        b'.' if rest.starts_with(b"..") => 2,
        b'-' if starts_any(rest, &[b"->", b"--", b"-="]) => 2,
        b':' if starts_any(rest, &[b"::", b":="]) => 2,
        b'&' if starts_any(rest, &[b"&&", b"&="]) => 2,
        b'|' if starts_any(rest, &[b"||", b"|="]) => 2,
        b'+' if starts_any(rest, &[b"++", b"+="]) => 2,
        b'*' if starts_any(rest, &[b"**", b"*="]) => 2,
        b'/' if starts_any(rest, &[b"//", b"/="]) => 2,
        b'%' if rest.starts_with(b"%=") => 2,
        b'^' if rest.starts_with(b"^=") => 2,
        b'?' if starts_any(rest, &[b"??", b"?."]) => 2,
        byte if byte.is_ascii() => 1,
        _ => char_len(source, cursor),
    };
    cursor + length
}

fn starts_any<const N: usize>(value: &[u8], candidates: &[&[u8]; N]) -> bool {
    for candidate in candidates {
        if value.starts_with(candidate) {
            return true;
        }
    }
    false
}

const fn is_identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'$')
}

const fn is_identifier_continue(byte: u8) -> bool {
    is_identifier_start(byte) || byte.is_ascii_digit()
}

fn char_len(source: &str, cursor: usize) -> usize {
    source[cursor..].chars().next().map_or(1, char::len_utf8)
}
