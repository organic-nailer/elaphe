use lrlex::lrlex_mod;
use lrpar::lrpar_mod;

lrlex_mod!("grammar.l");
lrpar_mod!("grammar.y");

pub use grammar_y::Node;

pub fn parse(source: &str) -> Option<Node> {
    let lexerdef = grammar_l::lexerdef();
    if source.trim().is_empty() {
        return None;
    }
    let lexer = lexerdef.lexer(source);
    let (res, _) = grammar_y::parse(&lexer);
    match res {
        Some(Ok(r)) => return Some(r),
        _ => return None
    };
}
