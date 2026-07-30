use crate::error::Result;
use crate::model::{Language, SourceFragment, SourceSpan};

pub(crate) fn fragment_file(
    path: &str,
    language: Language,
    text: &str,
    min_lines: usize,
    max_lines: usize,
) -> Result<Vec<SourceFragment>> {
    let offsets = line_offsets(text);
    let primary = match language {
        Language::Python => python_ranges(text, min_lines, max_lines),
        Language::Rust
        | Language::Go
        | Language::C
        | Language::Cpp
        | Language::Bash
        | Language::JavaScript
        | Language::TypeScript
        | Language::Java
        | Language::CSharp => brace_ranges(text, min_lines, max_lines),
        Language::Sql | Language::Markup | Language::Text => Vec::new(),
    };
    let primary = if primary.is_empty() {
        vec![(0, offsets.len().saturating_sub(1))]
    } else {
        primary
    };
    primary
        .into_iter()
        .filter(|(start, end)| end > start)
        .enumerate()
        .map(|(ordinal, (start, end))| {
            let start_byte = offsets[start];
            let end_byte = offsets.get(end + 1).copied().unwrap_or(text.len());
            SourceFragment::new(
                format!("{path}#fragment:{ordinal}"),
                path,
                language,
                SourceSpan {
                    start_byte,
                    end_byte,
                    start_line: u32::try_from(start + 1).unwrap_or(u32::MAX),
                    end_line: u32::try_from(end + 1).unwrap_or(u32::MAX),
                },
                &text[start_byte..end_byte],
            )
        })
        .collect()
}

fn line_offsets(text: &str) -> Vec<usize> {
    let mut offsets = vec![0];
    offsets.extend(
        text.match_indices('\n')
            .map(|(index, _)| index.saturating_add(1)),
    );
    offsets
}

fn python_ranges(text: &str, min_lines: usize, max_lines: usize) -> Vec<(usize, usize)> {
    let lines = text.lines().collect::<Vec<_>>();
    let mut ranges = Vec::new();
    for (start, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if !(trimmed.starts_with("def ") || trimmed.starts_with("async def ")) {
            continue;
        }
        let indentation = line.len() - trimmed.len();
        let mut end = start;
        for (index, candidate) in lines.iter().enumerate().skip(start + 1) {
            if candidate.trim().is_empty() {
                continue;
            }
            let candidate_indent = candidate.len() - candidate.trim_start().len();
            if candidate_indent <= indentation {
                break;
            }
            end = index;
            if end - start + 1 >= max_lines {
                break;
            }
        }
        if end.saturating_sub(start) + 1 >= min_lines {
            ranges.push((start, end));
        }
    }
    ranges
}

fn brace_ranges(text: &str, min_lines: usize, max_lines: usize) -> Vec<(usize, usize)> {
    let lines = text.lines().collect::<Vec<_>>();
    let mut ranges = Vec::new();
    let mut depth = 0_i32;
    let mut active = None::<(usize, i32)>;
    for (line_index, line) in lines.iter().enumerate() {
        let cleaned = strip_line_noise(line);
        let opens =
            i32::try_from(cleaned.bytes().filter(|byte| *byte == b'{').count()).unwrap_or(i32::MAX);
        let closes =
            i32::try_from(cleaned.bytes().filter(|byte| *byte == b'}').count()).unwrap_or(i32::MAX);
        if active.is_none() && depth <= 1 && opens > closes && looks_like_callable(&cleaned) {
            active = Some((line_index, depth));
        }
        depth += opens - closes;
        if let Some((start, parent_depth)) = active {
            let lines = line_index - start + 1;
            if depth <= parent_depth || lines >= max_lines {
                if lines >= min_lines {
                    ranges.push((start, line_index));
                }
                active = None;
            }
        }
    }
    ranges
}

fn strip_line_noise(line: &str) -> String {
    let mut output = String::with_capacity(line.len());
    let mut quote = None;
    let mut escaped = false;
    let mut chars = line.chars().peekable();
    while let Some(character) = chars.next() {
        if quote.is_none() && character == '/' && chars.peek() == Some(&'/') {
            break;
        }
        if quote.is_none() && character == '#' {
            break;
        }
        if escaped {
            escaped = false;
            continue;
        }
        if character == '\\' && quote.is_some() {
            escaped = true;
            continue;
        }
        if matches!(character, '\'' | '"' | '`') {
            if quote == Some(character) {
                quote = None;
            } else if quote.is_none() {
                quote = Some(character);
            }
            continue;
        }
        if quote.is_none() {
            output.push(character);
        }
    }
    output
}

fn looks_like_callable(line: &str) -> bool {
    let trimmed = line.trim_start();
    if [
        "if ", "if(", "for ", "for(", "while ", "while(", "switch ", "switch(", "match ", "catch ",
        "catch(", "else", "try ",
    ]
    .iter()
    .any(|prefix| trimmed.starts_with(prefix))
    {
        return false;
    }
    trimmed.contains('(')
        || ["fn ", "func ", "function "]
            .iter()
            .any(|marker| trimmed.contains(marker))
}

#[cfg(test)]
mod tests {
    use crate::fragment::fragment_file;
    use crate::model::Language;

    #[test]
    fn extracts_brace_and_python_functions() {
        let rust = "fn first() {\n let x = 1;\n println!(\"{x}\");\n}\n\nfn second() {\n let y = 2;\n println!(\"{y}\");\n}\n";
        assert_eq!(
            fragment_file("a.rs", Language::Rust, rust, 3, 100)
                .unwrap()
                .len(),
            2
        );
        let python =
            "def first():\n    x = 1\n    return x\n\ndef second():\n    y = 2\n    return y\n";
        assert_eq!(
            fragment_file("a.py", Language::Python, python, 3, 100)
                .unwrap()
                .len(),
            2
        );
    }
}
