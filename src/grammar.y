%start Expression
%%
Expression -> Result<Node, ()>:
    EqualityExpression { $1 };

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

CommaOpt -> Result<(), ()>:
      %empty { Ok(()) }
    | "," { Ok(()) }
    ;

Primary -> Result<Node, ()>:
      '(' Expression ')' { $2 }
    | Literal { $1 }
    | 'IDENTIFIER' { Ok(Node::Identifier { span: $span }) }
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
    }
}