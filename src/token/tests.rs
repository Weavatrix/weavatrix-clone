use super::{Interner, lex, tokenize};
use crate::config::CloneConfig;
use crate::model::Language;

#[test]
fn non_ascii_punctuation_always_advances() {
    let source = "fn пример() { let значение = 1; } — доказательство";
    let tokens = tokenize(
        source,
        Language::Rust,
        CloneConfig::default(),
        &mut Interner::default(),
    )
    .unwrap();
    assert!(!tokens.strict.is_empty());
    assert!(tokens.strict.len() < source.len());
}

#[test]
fn typescript_operators_and_numeric_signs_are_distinct_tokens() {
    let source = "const value = 1+2-3e-4; if (value !== 0 && value >>= 1) {}";
    let tokens = lex(source, Language::TypeScript, CloneConfig::default()).unwrap();
    let text = tokens
        .iter()
        .map(|token| &source[token.position.start_byte..token.position.end_byte])
        .collect::<Vec<_>>();
    assert!(text.windows(3).any(|part| part == ["1", "+", "2"]));
    assert!(text.contains(&"3e-4"));
    assert!(text.contains(&"!=="));
    assert!(text.contains(&"&&"));
    assert!(text.contains(&">>="));
}
