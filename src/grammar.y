%start AdditiveExpression
%%
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
      MultiplicativeExpression '*' PrimaryExpression { 
        Ok(Node::BinaryExpression { span: $span, operator: "*", left: Box::new($1?), right: Box::new($3?) })
    }
    | MultiplicativeExpression '/' PrimaryExpression { 
        Ok(Node::BinaryExpression { span: $span, operator: "/", left: Box::new($1?), right: Box::new($3?) })
    }
    | PrimaryExpression { $1 }
    ;

PrimaryExpression -> Result<Node, ()>:
      '(' AdditiveExpression ')' { $2 }
    | 'INT' { Ok(Node::NumericLiteral { span: $span }) }
    ;
%%
// Any functions here are in scope for all the grammar actions above.

use cfgrammar::Span;

#[derive(Debug)]
pub enum Node {
    BinaryExpression {
        span: Span,
        operator: &'static str,
        left: Box<Node>,
        right: Box<Node>,
    },
    NumericLiteral {
        span: Span,
    }
}