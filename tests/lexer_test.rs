use oxide::lexer::*;

#[test]
fn test_lexer_init() {
    let lang_def = definition_parser::load_language_definition(
        "~/Personal/Programming/Rust/oxide/src/lexer/rust_lang_def.toml",
    ).unwrap();

    let lexer = lexer::Lexer::new(
        &[
            "use oxide::lexer::*;",
            "fn main() {}",
        ],
        lang_def.clone(),
    );

    assert_eq!(lang_def, lexer.lang_def)
}
