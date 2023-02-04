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

#[cfg(test)]
mod tests {
    use lrlex::{LRNonStreamingLexer, DefaultLexerTypes};
    use lrpar::{Lexer, Lexeme};

    use super::*;

    fn lexer_result_to_vec<'a>(result: LRNonStreamingLexer< DefaultLexerTypes>, source: &'a str) -> Vec<&'a str> {
        result.iter().map(|r| {
            match r {
                Ok(lexeme) => {
                    let span = lexeme.span();
                    return &source[span.start()..span.end()];
                },
                Err(e) => {
                    panic!("Lex Error: {:?}", e);
                }
            }
        }).collect()
    }

    #[test]
    fn lexer() {
        let lexerdef = grammar_l::lexerdef();

        let source = "1 + 2.3*.9e+3/10.2e-20 + 0x2A";
        let result = lexerdef.lexer(source);
        let result: Vec<&str> = lexer_result_to_vec(result, source);
        assert_eq!(result, vec![
            "1","+","2.3","*",".9e+3","/","10.2e-20","+","0x2A"
        ]);

        let source = "'hoge ho123.4' + true + false +null";
        let result = lexerdef.lexer(source);
        let result: Vec<&str> = lexer_result_to_vec(result, source);
        assert_eq!(result, vec![
            "'hoge ho123.4'","+","true","+","false","+","null"
        ]);
    }
}
