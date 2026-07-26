use super::{CloneLocation, Language, Similarity, SourceFragment, SourceSpan};

#[test]
fn maps_every_language_profile_and_unknown_files() {
    for (path, expected) in [
        ("a.rs", Language::Rust),
        ("a.go", Language::Go),
        ("a.h", Language::C),
        ("a.hxx", Language::Cpp),
        ("a.zsh", Language::Bash),
        ("a.psql", Language::Sql),
        ("a.mjs", Language::JavaScript),
        ("a.mts", Language::TypeScript),
        ("a.pyi", Language::Python),
        ("a.java", Language::Java),
        ("a.cs", Language::CSharp),
        ("a.svelte", Language::Markup),
        ("Makefile", Language::Text),
    ] {
        assert_eq!(Language::from_path(path), expected);
    }
}

#[test]
fn validates_spans_paths_and_similarity_boundaries() {
    let text = "one\ntwo\n";
    let span = SourceSpan::whole(text);
    assert_eq!(span.end_line, 2);
    assert!(span.overlaps(SourceSpan {
        start_byte: 1,
        end_byte: 2,
        start_line: 1,
        end_line: 1,
    }));
    let fragment = SourceFragment::new("id", r"src\file.rs", Language::Rust, span, text).unwrap();
    assert_eq!(fragment.path, "src/file.rs");
    assert_eq!(CloneLocation::from_fragment(&fragment).path, fragment.path);
    assert!(SourceFragment::new("", "a.rs", Language::Rust, span, text).is_err());
    assert!(SourceFragment::new("id", "", Language::Rust, span, text).is_err());
    assert!(
        SourceFragment::new(
            "id",
            "a.rs",
            Language::Rust,
            SourceSpan {
                start_byte: 2,
                end_byte: 1,
                start_line: 1,
                end_line: 1,
            },
            text,
        )
        .is_err()
    );
    assert_eq!(Similarity::from_ratio(1, 0).permille(), 0);
    assert_eq!(Similarity::from_permille(1_500), Similarity::PERFECT);
    assert_eq!(Similarity::from_ratio(1, 2).permille(), 500);
}
