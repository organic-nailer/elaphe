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

const RESERVED_KEYWORDS: [&'static str; 33] = [
    "assert", "break", "case", "catch", "class", "const", "continue", "default", "do", "else",
    "enum", "extends", "false", "final", "finally", "for", "if", "in", "is", "new", "null",
    "rethrow", "return", "super", "switch", "this", "throw", "true", "try", "var", "void", "while",
    "with",
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
    let regex_boolean = Regex::new(r"^(true|false)").unwrap();
    let regex_identifier = Regex::new(r"^[a-zA-Z_\$][0-9a-zA-Z_\$]*").unwrap();

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

        match regex_boolean.find(&input[current_index..]) {
            Some(boolean) => {
                tokens.push(Token {
                    kind: TokenKind::Boolean,
                    str: &input[current_index..current_index + boolean.end()],
                });
                current_index += boolean.end();
                continue 'tokenize;
            }
            None => {}
        }

        if input[current_index..].starts_with("null") {
            tokens.push(Token {
                kind: TokenKind::Null,
                str: "null",
            });
            current_index += 4;
            continue 'tokenize;
        }

        for keyword in RESERVED_KEYWORDS.iter() {
            if input[current_index..].starts_with(keyword) {
                tokens.push(Token {
                    kind: TokenKind::Keyword,
                    str: keyword,
                });
                current_index += keyword.len();
                continue 'tokenize;
            }
        }

        for identifier in BUILT_IN_IDENTIFIER.iter() {
            if input[current_index..].starts_with(identifier) {
                tokens.push(Token {
                    kind: TokenKind::BuiltInIdentifier,
                    str: identifier,
                });
                current_index += identifier.len();
                continue 'tokenize;
            }
        }

        for identifier in OTHER_IDENTIFIER.iter() {
            if input[current_index..].starts_with(identifier) {
                tokens.push(Token {
                    kind: TokenKind::OtherIdentifier,
                    str: identifier,
                });
                current_index += identifier.len();
                continue 'tokenize;
            }
        }

        match regex_identifier.find(&input[current_index..]) {
            Some(identifier) => {
                tokens.push(Token {
                    kind: TokenKind::Identifier,
                    str: &input[current_index..current_index + identifier.end()],
                });
                current_index += identifier.end();
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
    }
}
