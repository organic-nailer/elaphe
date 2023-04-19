use dart_parser_generator::grammar::END;
use regex::Regex;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TokenKind {
    Number,
    String,
    Boolean,
    Null,
    Keyword,
    Identifier,
    BuiltInIdentifier,
    OtherIdentifier,
    Symbol,
    EOF,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token<'input> {
    pub kind: TokenKind,
    pub str: &'input str,
}

impl Token<'_> {
    pub fn kind_str(&self) -> String {
        match self.kind {
            TokenKind::Keyword
            | TokenKind::Symbol
            | TokenKind::BuiltInIdentifier
            | TokenKind::OtherIdentifier => self.str.to_string(),
            TokenKind::Number => String::from("NUMBER"),
            TokenKind::String => String::from("STRING"),
            TokenKind::Boolean => String::from("BOOLEAN"),
            TokenKind::Null => String::from("NULL"),
            TokenKind::Identifier => String::from("IDENTIFIER"),
            TokenKind::EOF => String::from(END),
        }
    }
}

const RESERVED_KEYWORDS: [&'static str; 34] = [
    "assert", "break", "case", "catch", "class", "const", "continue", "default", "do", "else",
    "enum", "extends", "false", "final", "finally", "for", "if", "in", "is", "new", "null",
    "rethrow", "return", "super", "switch", "this", "throw", "true", "try", "var", "void", "while",
    "with", "sl",
];

const BUILT_IN_IDENTIFIER: [&'static str; 23] = [
    "abstract",
    "as",
    "covariant",
    "deferred",
    "dynamic",
    "export",
    "external",
    "extension",
    "factory",
    "Function",
    "get",
    "implements",
    "import",
    "interface",
    "late",
    "library",
    "mixin",
    "operator",
    "part",
    "required",
    "set",
    "static",
    "typedef",
];

const OTHER_IDENTIFIER: [&'static str; 8] = [
    "async", "hide", "of", "on", "show", "sync", "await", "yield",
];

const SYMBOLS: [&'static str; 49] = [
    "<<=", ">>=", "??=", "~/=", "??", "&&", "||", "==", "!=", "<<", ">>", ">=", "<=", "*=", "/=",
    "%=", "+=", "-=", "&=", "^=", "|=", "=>", "~/", "++", "--", "?", ":", ">", ";", "=", "{", "}",
    "<", "!", "~", "|", "^", "&", "+", "-", "*", "/", "%", "(", ")", ",", ".", "[", "]",
];

pub fn tokenize<'input>(input: &'input str) -> Vec<Token<'input>> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut current_index = 0;

    let regex_whitespace = Regex::new(r"^[\t\n\r ]+").unwrap();
    let regex_single_comment = Regex::new(r"^//[^\n]*").unwrap();
    let regex_multi_comment = Regex::new(r"^/\*(.|\n)*?\*/").unwrap();
    let regex_string = Regex::new(r#"^(('[^\\'$]*')|("[^\\"$]*"))"#).unwrap();
    let regex_number = Regex::new(r"^((0(x|X)[a-fA-F0-9]+)|((([0-9]+(\.[0-9]+)?((e|E)(\+|-)?[0-9]+)?)|(\.[0-9]+((e|E)(\+|-)?[0-9]+)?))))").unwrap();
    let regex_identifier_or_keyword = Regex::new(r"^[a-zA-Z_\$][0-9a-zA-Z_\$]*").unwrap();

    'tokenize: loop {
        if current_index >= input.len() {
            break;
        }

        match regex_whitespace.find(&input[current_index..]) {
            Some(whitespace) => {
                current_index += whitespace.end();
                continue 'tokenize;
            }
            None => {}
        }

        match regex_single_comment.find(&input[current_index..]) {
            Some(single_comment) => {
                current_index += single_comment.end();
                continue 'tokenize;
            }
            None => {}
        }

        match regex_multi_comment.find(&input[current_index..]) {
            Some(multi_comment) => {
                current_index += multi_comment.end();
                continue 'tokenize;
            }
            None => {}
        }

        match regex_string.find(&input[current_index..]) {
            Some(string) => {
                tokens.push(Token {
                    kind: TokenKind::String,
                    str: &input[current_index..current_index + string.end()],
                });
                current_index += string.end();
                continue 'tokenize;
            }
            None => {}
        }

        match regex_number.find(&input[current_index..]) {
            Some(number) => {
                tokens.push(Token {
                    kind: TokenKind::Number,
                    str: &input[current_index..current_index + number.end()],
                });
                current_index += number.end();
                continue 'tokenize;
            }
            None => {}
        }

        match regex_identifier_or_keyword.find(&input[current_index..]) {
            Some(identifier_or_keyword) => {
                let identifier_or_keyword =
                    &input[current_index..current_index + identifier_or_keyword.end()];
                if identifier_or_keyword == "true" || identifier_or_keyword == "false" {
                    tokens.push(Token {
                        kind: TokenKind::Boolean,
                        str: identifier_or_keyword,
                    });
                    current_index += identifier_or_keyword.len();
                    continue 'tokenize;
                }
                if identifier_or_keyword == "null" {
                    tokens.push(Token {
                        kind: TokenKind::Null,
                        str: identifier_or_keyword,
                    });
                    current_index += identifier_or_keyword.len();
                    continue 'tokenize;
                }
                if RESERVED_KEYWORDS.contains(&identifier_or_keyword) {
                    tokens.push(Token {
                        kind: TokenKind::Keyword,
                        str: identifier_or_keyword,
                    });
                    current_index += identifier_or_keyword.len();
                    continue 'tokenize;
                }
                if BUILT_IN_IDENTIFIER.contains(&identifier_or_keyword) {
                    tokens.push(Token {
                        kind: TokenKind::BuiltInIdentifier,
                        str: identifier_or_keyword,
                    });
                    current_index += identifier_or_keyword.len();
                    continue 'tokenize;
                }
                if OTHER_IDENTIFIER.contains(&identifier_or_keyword) {
                    tokens.push(Token {
                        kind: TokenKind::OtherIdentifier,
                        str: identifier_or_keyword,
                    });
                    current_index += identifier_or_keyword.len();
                    continue 'tokenize;
                }
                tokens.push(Token {
                    kind: TokenKind::Identifier,
                    str: identifier_or_keyword,
                });
                current_index += identifier_or_keyword.len();
                continue 'tokenize;
            }
            None => {}
        }

        for symbol in SYMBOLS.iter() {
            if input[current_index..].starts_with(symbol) {
                tokens.push(Token {
                    kind: TokenKind::Symbol,
                    str: symbol,
                });
                current_index += symbol.len();
                continue 'tokenize;
            }
        }

        panic!("Unexpected token at {}", current_index);
    }
    tokens.push(Token {
        kind: TokenKind::EOF,
        str: "",
    });
    tokens
}

#[cfg(test)]
mod tests {
    use crate::tokenizer::{tokenize, Token, TokenKind};

    #[test]
    fn lexer() {
        let source = "1 + 2.3*.9e+3/10.2e-20 + 0x2A";
        let result = tokenize(source);
        assert_eq!(
            result,
            vec![
                Token {
                    kind: TokenKind::Number,
                    str: "1",
                },
                Token {
                    kind: TokenKind::Symbol,
                    str: "+",
                },
                Token {
                    kind: TokenKind::Number,
                    str: "2.3",
                },
                Token {
                    kind: TokenKind::Symbol,
                    str: "*",
                },
                Token {
                    kind: TokenKind::Number,
                    str: ".9e+3",
                },
                Token {
                    kind: TokenKind::Symbol,
                    str: "/",
                },
                Token {
                    kind: TokenKind::Number,
                    str: "10.2e-20",
                },
                Token {
                    kind: TokenKind::Symbol,
                    str: "+",
                },
                Token {
                    kind: TokenKind::Number,
                    str: "0x2A",
                },
                Token {
                    kind: TokenKind::EOF,
                    str: ""
                }
            ]
        );

        let source = "'hoge ho123.4' + true + false +null";
        let result = tokenize(source);
        assert_eq!(
            result,
            vec![
                Token {
                    kind: TokenKind::String,
                    str: "'hoge ho123.4'",
                },
                Token {
                    kind: TokenKind::Symbol,
                    str: "+",
                },
                Token {
                    kind: TokenKind::Boolean,
                    str: "true",
                },
                Token {
                    kind: TokenKind::Symbol,
                    str: "+",
                },
                Token {
                    kind: TokenKind::Boolean,
                    str: "false",
                },
                Token {
                    kind: TokenKind::Symbol,
                    str: "+",
                },
                Token {
                    kind: TokenKind::Null,
                    str: "null",
                },
                Token {
                    kind: TokenKind::EOF,
                    str: ""
                }
            ]
        );

        let source = "var truely = true; as finally";
        let result = tokenize(source);
        assert_eq!(
            result,
            vec![
                Token {
                    kind: TokenKind::Keyword,
                    str: "var",
                },
                Token {
                    kind: TokenKind::Identifier,
                    str: "truely",
                },
                Token {
                    kind: TokenKind::Symbol,
                    str: "=",
                },
                Token {
                    kind: TokenKind::Boolean,
                    str: "true",
                },
                Token {
                    kind: TokenKind::Symbol,
                    str: ";",
                },
                Token {
                    kind: TokenKind::BuiltInIdentifier,
                    str: "as",
                },
                Token {
                    kind: TokenKind::Keyword,
                    str: "finally",
                },
                Token {
                    kind: TokenKind::EOF,
                    str: ""
                }
            ]
        );
    }
}
