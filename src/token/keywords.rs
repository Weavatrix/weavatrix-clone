use crate::model::Language;

pub(super) fn is_keyword(token: &str, language: Language) -> bool {
    if matches!(
        token,
        "if" | "else"
            | "for"
            | "while"
            | "loop"
            | "match"
            | "switch"
            | "case"
            | "break"
            | "continue"
            | "return"
            | "throw"
            | "try"
            | "catch"
            | "finally"
            | "new"
            | "class"
            | "struct"
            | "enum"
            | "interface"
            | "type"
            | "import"
            | "export"
            | "from"
            | "async"
            | "await"
            | "yield"
            | "true"
            | "false"
            | "null"
            | "nil"
            | "this"
            | "self"
    ) {
        return true;
    }
    match language {
        Language::Rust => matches!(
            token,
            "fn" | "let" | "mut" | "impl" | "trait" | "pub" | "crate" | "super" | "where"
        ),
        Language::Go => matches!(
            token,
            "func" | "var" | "const" | "package" | "defer" | "go" | "chan" | "range" | "select"
        ),
        Language::Python => matches!(
            token,
            "def" | "lambda" | "elif" | "except" | "with" | "as" | "pass" | "raise" | "in" | "is"
        ),
        Language::Sql => matches!(
            token.to_ascii_lowercase().as_str(),
            "select"
                | "insert"
                | "update"
                | "delete"
                | "from"
                | "where"
                | "join"
                | "group"
                | "order"
                | "having"
                | "create"
                | "alter"
                | "drop"
        ),
        _ => false,
    }
}
