use crate::{CloneConfig, CloneError, Language, Result};
use std::collections::HashMap;

mod keywords;
mod lexer;
#[cfg(test)]
mod tests;

use keywords::is_keyword;
use lexer::Kind;
pub(crate) use lexer::lex;

const IDENTIFIER: &str = "\0identifier";
const STRING_LITERAL: &str = "\0string";

#[derive(Debug, Default)]
pub(crate) struct Interner {
    ids: HashMap<String, u32>,
}

impl Interner {
    pub(crate) fn intern(&mut self, value: &str) -> Result<u32> {
        if let Some(id) = self.ids.get(value) {
            return Ok(*id);
        }
        let id = u32::try_from(self.ids.len()).map_err(|_| CloneError::CapacityExceeded {
            resource: "unique tokens",
            limit: u32::MAX as usize,
        })?;
        self.ids.insert(value.to_owned(), id);
        Ok(id)
    }
}

#[derive(Debug)]
pub(crate) struct Tokenized {
    pub strict: Vec<u32>,
    pub renamed: Vec<u32>,
}

#[cfg_attr(not(feature = "scan"), allow(dead_code))]
#[derive(Debug, Clone, Copy)]
pub(crate) struct TokenPosition {
    pub start_byte: usize,
    pub end_byte: usize,
    pub start_line: u32,
    pub end_line: u32,
}

pub(crate) fn tokenize(
    source: &str,
    language: Language,
    config: CloneConfig,
    interner: &mut Interner,
) -> Result<Tokenized> {
    let lexed = lex(source, language, config)?;
    let mut strict = Vec::with_capacity(lexed.len());
    let mut renamed = Vec::with_capacity(lexed.len());
    for token in lexed {
        let raw = &source[token.position.start_byte..token.position.end_byte];
        let strict_id = interner.intern(raw)?;
        let renamed_id = match token.kind {
            Kind::Identifier if !is_keyword(raw, language) => interner.intern(IDENTIFIER)?,
            Kind::String => interner.intern(STRING_LITERAL)?,
            Kind::Identifier | Kind::Number | Kind::Syntax => strict_id,
        };
        strict.push(strict_id);
        renamed.push(renamed_id);
    }
    Ok(Tokenized { strict, renamed })
}
