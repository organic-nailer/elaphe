%start Statement
%expect 1
%%
Statements -> Result<Vec<Box<Node>>, ()>:
      %empty { Ok(vec![]) }
    | Statements Statement { flatten($1, $2?) }
    ;

Statement -> Result<Node, ()>:
      BlockStatement { $1 }
    | LocalVariableDeclaration { $1 }
    | IfStatement { $1 }
    | ForStatement { $1 }
    | WhileStatement { $1 }
    | DoStatement { $1 }
    | ExpressionStatement { $1 }
    | ";" { Ok(Node::EmptyStatement { span: $span }) }
    ;

BlockStatement -> Result<Node, ()>:
      "{" Statements "}" { Ok(Node::BlockStatement { span: $span, children: $2? }) }
    ;

LocalVariableDeclaration -> Result<Node, ()>:
    InitializedVariableDeclaration ";" { $1 };

ExpressionStatement -> Result<Node, ()>:
    Expression ";" { Ok(Node::ExpressionStatement { span: $span, expr: Box::new($1?) }) }
    ;

IfStatement -> Result<Node, ()>:
      "if" "(" Expression ")" Statement { Ok(Node::IfStatement { span: $span, condition: Box::new($3?), if_true_stmt: Box::new($5?), if_false_stmt: None }) }
    | "if" "(" Expression ")" Statement "else" Statement { Ok(Node::IfStatement { span: $span, condition: Box::new($3?), if_true_stmt: Box::new($5?), if_false_stmt: Some(Box::new($7?)) }) }
    ;

ForStatement -> Result<Node, ()>:
    "for" "(" ForLoopParts ")" Statement {
        let part = $3?;
        Ok(Node::ForStatement { span: $span, init: part.0, condition: part.1, update: part.2, stmt: Box::new($5?) })
    }
    ;

ForLoopParts -> Result<(Option<Box<Node>>,Option<Box<Node>>,Option<Vec<Box<Node>>>), ()>:
      ForInitializerStatement ExpressionOpt ";" ExpressionListOpt {
        Ok(($1?, $2?, $4?))
      }
    ;

ForInitializerStatement -> Result<Option<Box<Node>>, ()>:
      LocalVariableDeclaration { Ok(Some(Box::new($1?))) }
    | ExpressionOpt ";" {
        match $1? {
            Some(v) => Ok(Some(Box::new(Node::ExpressionStatement { span: $span, expr: v }))),
            None => Ok(None),
        }
     }
    ;

WhileStatement -> Result<Node, ()>:
    "while" "(" Expression ")" Statement {
        Ok(Node::WhileStatement { span: $span, condition: Box::new($3?), stmt: Box::new($5?) })
    }
    ;

DoStatement -> Result<Node, ()>:
    "do" Statement "while" "(" Expression ")" ";" {
        Ok(Node::DoStatement { span: $span, condition: Box::new($5?), stmt: Box::new($2?) })
    }
    ;

InitializedVariableDeclaration -> Result<Node, ()>:
      DeclaredIdentifier { Ok(Node::VariableDeclaration { span: $span, identifier: Box::new($1?), expr: None }) }
    | DeclaredIdentifier "=" Expression { Ok(Node::VariableDeclaration { span: $span, identifier: Box::new($1?), expr: Some(Box::new($3?)) }) }
    ;

DeclaredIdentifier -> Result<Node, ()>:
    "var" Identifier { $2 }
    ;





Expression -> Result<Node, ()>:
    EqualityExpression { $1 };

ExpressionOpt -> Result<Option<Box<Node>>, ()>:
      %empty { Ok(None) }
    | Expression { Ok(Some(Box::new($1?))) }
    ;

EqualityExpression -> Result<Node, ()>:
      RelationalExpression "==" RelationalExpression {
        Ok(Node::BinaryExpression { span: $span, operator: "==", left: Box::new($1?), right: Box::new($3?) })
    }
    | RelationalExpression "!=" RelationalExpression {
        Ok(Node::BinaryExpression { span: $span, operator: "!=", left: Box::new($1?), right: Box::new($3?) })
    }
    | RelationalExpression { $1 }
    ;

RelationalExpression -> Result<Node, ()>:
      BitwiseOrExpression ">=" BitwiseOrExpression {
        Ok(Node::BinaryExpression { span: $span, operator: ">=", left: Box::new($1?), right: Box::new($3?) })
    }
    | BitwiseOrExpression ">" BitwiseOrExpression {
        Ok(Node::BinaryExpression { span: $span, operator: ">", left: Box::new($1?), right: Box::new($3?) })
    }
    | BitwiseOrExpression "<=" BitwiseOrExpression {
        Ok(Node::BinaryExpression { span: $span, operator: "<=", left: Box::new($1?), right: Box::new($3?) })
    }
    | BitwiseOrExpression "<" BitwiseOrExpression {
        Ok(Node::BinaryExpression { span: $span, operator: "<", left: Box::new($1?), right: Box::new($3?) })
    }
    | BitwiseOrExpression { $1 }
    ;

BitwiseOrExpression -> Result<Node, ()>:
      BitwiseOrExpression "|" BitwiseXorExpression {
        Ok(Node::BinaryExpression { span: $span, operator: "|", left: Box::new($1?), right: Box::new($3?) })
    }
    | BitwiseXorExpression { $1 }
    ;

BitwiseXorExpression -> Result<Node, ()>:
      BitwiseXorExpression "^" BitwiseAndExpression {
        Ok(Node::BinaryExpression { span: $span, operator: "^", left: Box::new($1?), right: Box::new($3?) })
    }
    | BitwiseAndExpression { $1 }
    ;

BitwiseAndExpression -> Result<Node, ()>:
      BitwiseAndExpression "&" ShiftExpression {
        Ok(Node::BinaryExpression { span: $span, operator: "&", left: Box::new($1?), right: Box::new($3?) })
    }
    | ShiftExpression { $1 }
    ;

ShiftExpression -> Result<Node, ()>:
      ShiftExpression "<<" AdditiveExpression {
        Ok(Node::BinaryExpression { span: $span, operator: "<<", left: Box::new($1?), right: Box::new($3?) })
    }
    | ShiftExpression ">>" AdditiveExpression {
        Ok(Node::BinaryExpression { span: $span, operator: ">>", left: Box::new($1?), right: Box::new($3?) })
    }
    | AdditiveExpression { $1 }
    ;

AdditiveExpression -> Result<Node, ()>:
      AdditiveExpression '+' MultiplicativeExpression { 
        Ok(Node::BinaryExpression { span: $span, operator: "+", left: Box::new($1?), right: Box::new($3?) })
    }
    | AdditiveExpression '-' MultiplicativeExpression { 
        Ok(Node::BinaryExpression { span: $span, operator: "-", left: Box::new($1?), right: Box::new($3?) })
    }
    | MultiplicativeExpression { $1 }
    ;

MultiplicativeExpression -> Result<Node, ()>:
      MultiplicativeExpression '*' Primary { 
        Ok(Node::BinaryExpression { span: $span, operator: "*", left: Box::new($1?), right: Box::new($3?) })
    }
    | MultiplicativeExpression '/' Primary { 
        Ok(Node::BinaryExpression { span: $span, operator: "/", left: Box::new($1?), right: Box::new($3?) })
    }
    | UnaryExpression { $1 }
    ;

UnaryExpression -> Result<Node, ()>:
      "-" UnaryExpression {
        Ok(Node::UnaryOpExpression { span: $span, operator: "-", child: Box::new($2?) })
    }
    | "!" UnaryExpression {
        Ok(Node::UnaryOpExpression { span: $span, operator: "!", child: Box::new($2?) })
    }
    | "~" UnaryExpression {
        Ok(Node::UnaryOpExpression { span: $span, operator: "~", child: Box::new($2?) })
    }
    | PostfixExpression { $1 }
    ;

PostfixExpression -> Result<Node, ()>:
      PostfixExpression Selector {
        Ok(Node::WithSelectorExpression { span: $span, child: Box::new($1?), selector: Box::new($2?) })
    }
    | Primary { $1 }
    ;

Selector -> Result<Node, ()>:
      Arguments { $1 };

Arguments -> Result<Node, ()>:
      "(" ")" { Ok(Node::Arguments { span: $span, children: vec![] }) }
    | "(" ExpressionList CommaOpt ")" { Ok(Node::Arguments { span: $span, children: $2? }) }
    ;

ExpressionList -> Result<Vec<Box<Node>>, ()>:
      ExpressionList "," Expression { 
        flatten($1, $3?)
    }
    | Expression { Ok(vec![Box::new($1?)]) }
    ;

ExpressionListOpt -> Result<Option<Vec<Box<Node>>>, ()>:
      %empty { Ok(None) }
    | ExpressionList { Ok(Some($1?)) }
    ;

CommaOpt -> Result<(), ()>:
      %empty { Ok(()) }
    | "," { Ok(()) }
    ;

Primary -> Result<Node, ()>:
      '(' Expression ')' { $2 }
    | Literal { $1 }
    | Identifier { $1 }
    ;

Identifier -> Result<Node, ()>:
    'IDENTIFIER' { Ok(Node::Identifier { span: $span }) }
    ;

Literal -> Result<Node, ()>:
      'NUMBER' { Ok(Node::NumericLiteral { span: $span }) }
    | 'STRING' { Ok(Node::StringLiteral { span: $span }) }
    | 'BOOLEAN' { Ok(Node::BooleanLiteral { span: $span }) }
    | 'NULL' { Ok(Node::NullLiteral { span: $span }) }
    ;
%%
// Any functions here are in scope for all the grammar actions above.

fn flatten<T>(left: Result<Vec<Box<T>>,()>, right: T) -> Result<Vec<Box<T>>,()> {
    let mut flt = left?;
    flt.push(Box::new(right));
    Ok(flt)
}

use cfgrammar::Span;

#[derive(Debug)]
pub enum Node {
    BinaryExpression {
        span: Span,
        operator: &'static str,
        left: Box<Node>,
        right: Box<Node>,
    },
    UnaryOpExpression {
        span: Span,
        operator: &'static str,
        child: Box<Node>,
    },
    NumericLiteral {
        span: Span,
    },
    StringLiteral {
        span: Span,
    },
    BooleanLiteral {
        span: Span,
    },
    NullLiteral {
        span: Span,
    },
    Identifier {
        span: Span,
    },
    Arguments {
        span: Span,
        children: Vec<Box<Node>>
    },
    WithSelectorExpression {
        span: Span,
        child: Box<Node>,
        selector: Box<Node>,
    },

    BlockStatement {
        span: Span,
        children: Vec<Box<Node>>,
    },
    ExpressionStatement {
        span: Span,
        expr: Box<Node>,
    },
    EmptyStatement {
        span: Span,
    },
    VariableDeclaration {
        span: Span,
        identifier: Box<Node>,
        expr: Option<Box<Node>>,
    },
    IfStatement {
        span: Span,
        condition: Box<Node>,
        if_true_stmt: Box<Node>,
        if_false_stmt: Option<Box<Node>>,
    },
    ForStatement {
        span: Span,
        init: Option<Box<Node>>,
        condition: Option<Box<Node>>,
        update: Option<Vec<Box<Node>>>,
        stmt: Box<Node>,
    },
    WhileStatement {
        span: Span,
        condition: Box<Node>,
        stmt: Box<Node>,
    },
    DoStatement {
        span: Span,
        condition: Box<Node>,
        stmt: Box<Node>,
    }
}