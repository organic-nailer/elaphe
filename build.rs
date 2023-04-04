use cfgrammar::yacc::YaccKind;
use copy_to_output::copy_to_output;
use lrlex::CTLexerBuilder;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build the lexer and parser.
    CTLexerBuilder::new()
        .lrpar_config(|ctp| {
            ctp.yacckind(YaccKind::Grmtools)
                .grammar_in_src_dir("grammar.y")
                .unwrap()
        })
        .lexer_in_src_dir("grammar.l")?
        .build()?;

    copy_to_output("template", &env::var("PROFILE").unwrap())
        .expect("failed to copy template files");
    copy_to_output("script", &env::var("PROFILE").unwrap()).expect("failed to copy script files");
    Ok(())
}
