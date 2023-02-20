%start LibraryDeclaration
%expect 2
%%

LibraryDeclaration -> Result<LibraryDeclaration, ()>:
    LibraryImportList TopLevelDeclarationList { 
        Ok(LibraryDeclaration { import_list: $1?, top_level_declaration_list: $2? })
    }
    ;

LibraryImportList -> Result<Vec<LibraryImport>, ()>:
      %empty { Ok(vec![]) }
    | LibraryImportList LibraryImport { flatten($1, $2?) }
    ;

LibraryImport -> Result<LibraryImport, ()>:
      "import" Uri ";" { Ok(LibraryImport { uri: $2?, identifier: None }) }
    | "import" Uri "as" Identifier ";" { Ok(LibraryImport { uri: $2?, identifier: Some(Box::new($4?)) }) }
    ;

Uri -> Result<Span, ()>:
    "STRING" { Ok($span) }
    ;

TopLevelDeclarationList -> Result<Vec<Box<Node>>, ()>:
      %empty { Ok(vec![]) }
    | TopLevelDeclarationList TopLevelDeclaration { flatten($1, Box::new($2?)) }
    ;

TopLevelDeclaration -> Result<Node, ()>:
      TopFunctionDeclaration { $1 }
    | TopVariableDeclaration { $1 }
    ;

TopFunctionDeclaration -> Result<Node, ()>:
      Identifier FormalParameterList FunctionBody {
        Ok(Node::FunctionDeclaration { span: $span, identifier: Box::new($1?), parameters: $2?, body: Box::new($3?) })
    }
    ;

FormalParameterList -> Result<Vec<FunctionParameter>, ()>:
      "(" ")" { Ok(vec![]) }
    | "(" NormalFormalParameterList CommaOpt ")" { $2 }
    ;

NormalFormalParameterList -> Result<Vec<FunctionParameter>, ()>:
      NormalFormalParameter { Ok(vec![$1?]) }
    | NormalFormalParameterList "," NormalFormalParameter { flatten($1, $3?) }
    ;

NormalFormalParameter -> Result<FunctionParameter, ()>:
      Identifier { Ok(FunctionParameter { identifier: Box::new($1?) }) }
    ;

FunctionBody -> Result<Node, ()>:
      "=>" Expression ";" { Ok(Node::ExpressionStatement { span: $span, expr: Box::new($2?) }) }
    | BlockStatement { $1 }
    ;

TopVariableDeclaration -> Result<Node, ()>:
      "var" Identifier ";" { Ok(Node::VariableDeclaration { span: $span, identifier: Box::new($2?), expr: None }) }
    | "var" Identifier "=" Expression ";" {
        Ok(Node::VariableDeclaration { span: $span, identifier: Box::new($2?), expr: Some(Box::new($4?)) })
    }
    ;









Statements -> Result<Vec<Box<Node>>, ()>:
      %empty { Ok(vec![]) }
    | Statements Statement { flatten($1, Box::new($2?)) }
    ;

Statement -> Result<Node, ()>:
      NonLabeledStatement { $1 }
    | Label NonLabeledStatement {
        Ok(Node::LabeledStatement { span: $span, label: $1?, stmt: Box::new($2?) })
    }
    ;

NonLabeledStatement -> Result<Node, ()>:
      BlockStatement { $1 }
    | LocalVariableDeclaration { $1 }
    | IfStatement { $1 }
    | ForStatement { $1 }
    | WhileStatement { $1 }
    | DoStatement { $1 }
    | BreakStatement { $1 }
    | ContinueStatement { $1 }
    | ReturnStatement { $1 }
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

Label -> Result<StatementLabel, ()>:
    Identifier ":" { Ok(StatementLabel { identifier: Box::new($1?) }) }
    ;

BreakStatement -> Result<Node, ()>:
      "break" ";" {
        Ok(Node::BreakStatement { span: $span, label: None })
    }
    | "break" Identifier ";" {
        Ok(Node::BreakStatement { span: $span, label: Some(Box::new($2?)) })
    }
    ;

ContinueStatement -> Result<Node, ()>:
      "continue" ";" {
        Ok(Node::ContinueStatement { span: $span, label: None })
    }
    | "continue" Identifier ";" {
        Ok(Node::ContinueStatement { span: $span, label: Some(Box::new($2?)) })
    }
    ;

ReturnStatement -> Result<Node, ()>:
      "return" ";" { Ok(Node::ReturnStatement { span: $span, value: None }) }
    | "return" Expression ";" {
        Ok(Node::ReturnStatement { span: $span, value: Some(Box::new($2?)) })
    }
    ;






Expression -> Result<Node, ()>:
      AssignableExpression AssignmentOperator Expression {
        Ok(Node::AssignmentExpression { span: $span, operator: $2?, left: Box::new($1?), right: Box::new($3?) })
    }
    | ConditionalExpression { $1 }
    ;

AssignableExpression -> Result<Node, ()>:
      Identifier { $1 }
    ;

AssignmentOperator -> Result<&'static str, ()>:
      "=" { Ok("=") }
    | "*=" { Ok("*=") }
    | "/=" { Ok("/=") }
    | "~/=" { Ok("~/=") }
    | "%=" { Ok("%=") }
    | "+=" { Ok("+=") }
    | "-=" { Ok("-=") }
    | "<<=" { Ok("<<=") }
    | ">>=" { Ok(">>=") }
    | "&=" { Ok("&=") }
    | "^=" { Ok("^=") }
    | "|=" { Ok("|=") }
    | "??=" { Ok("??=") }
    ;

ExpressionOpt -> Result<Option<Box<Node>>, ()>:
      %empty { Ok(None) }
    | Expression { Ok(Some(Box::new($1?))) }
    ;

ConditionalExpression -> Result<Node, ()>:
      IfNullExpression { $1 }
    | IfNullExpression "?" Expression ":" Expression {
        Ok(Node::ConditionalExpression { span: $span, condition: Box::new($1?), if_true_expr: Box::new($3?), if_false_expr: Box::new($5?) })
    }
    ;

IfNullExpression -> Result<Node, ()>:
      LogicalOrExpression { $1 }
    | IfNullExpression "??" LogicalOrExpression{
        Ok(Node::BinaryExpression { span: $span, operator: "??", left: Box::new($1?), right: Box::new($3?) })
    }
    ;

LogicalOrExpression -> Result<Node, ()>:
      LogicalAndExpression { $1 }
    | LogicalOrExpression "||" LogicalAndExpression{
        Ok(Node::BinaryExpression { span: $span, operator: "||", left: Box::new($1?), right: Box::new($3?) })
    }
    ;

LogicalAndExpression -> Result<Node, ()>:
      EqualityExpression { $1 }
    | LogicalAndExpression "&&" EqualityExpression{
        Ok(Node::BinaryExpression { span: $span, operator: "&&", left: Box::new($1?), right: Box::new($3?) })
    }
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
      Arguments { $1 }
    | "." Identifier { Ok(Node::SelectorAttr { span: $span, identifier: Box::new($2?) }) }
    | "." Identifier Arguments {
        Ok(Node::SelectorMethod { span: $span, identifier: Box::new($2?), arguments: Box::new($3?) })
    }
    ;

Arguments -> Result<Node, ()>:
      "(" ")" { Ok(Node::Arguments { span: $span, children: vec![] }) }
    | "(" ExpressionList CommaOpt ")" { Ok(Node::Arguments { span: $span, children: $2? }) }
    ;

ExpressionList -> Result<Vec<Box<Node>>, ()>:
      ExpressionList "," Expression { 
        flatten($1, Box::new($3?))
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
    | StringLiteralList { Ok(Node::StringLiteral { span: $span, literal_list: $1? }) }
    | 'BOOLEAN' { Ok(Node::BooleanLiteral { span: $span }) }
    | 'NULL' { Ok(Node::NullLiteral { span: $span }) }
    ;

StringLiteralList -> Result<Vec<Span>, ()>:
      StringLiteralList "STRING" { 
        match $2 {
            Ok(v) => flatten($1, v.span()),
            Err(_) => Err(())
        }
    }
    | "STRING" { 
        match $1 {
            Ok(v) => Ok(vec![v.span()]),
            Err(_) => Err(())
        }
    }
    ;
%%
// Any functions here are in scope for all the grammar actions above.
















fn flatten<T>(left: Result<Vec<T>,()>, right: T) -> Result<Vec<T>,()> {
    let mut flt = left?;
    flt.push(right);
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
    ConditionalExpression {
        span: Span,
        condition: Box<Node>,
        if_true_expr: Box<Node>,
        if_false_expr: Box<Node>,
    },
    UnaryOpExpression {
        span: Span,
        operator: &'static str,
        child: Box<Node>,
    },
    AssignmentExpression {
        span: Span,
        operator: &'static str,
        left: Box<Node>,
        right: Box<Node>,
    },
    NumericLiteral {
        span: Span,
    },
    StringLiteral {
        span: Span,
        literal_list: Vec<Span>,
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

    LabeledStatement {
        span: Span,
        label: StatementLabel,
        stmt: Box<Node>,
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
    },
    BreakStatement {
        span: Span,
        label: Option<Box<Node>>,
    },
    ContinueStatement {
        span: Span,
        label: Option<Box<Node>>,
    },
    ReturnStatement {
        span: Span,
        value: Option<Box<Node>>,
    },
    FunctionDeclaration {
        span: Span,
        identifier: Box<Node>,
        parameters: Vec<FunctionParameter>,
        body: Box<Node>,
    },
    SelectorAttr {
        span: Span,
        identifier: Box<Node>,
    },
    SelectorMethod {
        span: Span,
        identifier: Box<Node>,
        arguments: Box<Node>,
    }
}

#[derive(Debug)]
pub struct LibraryDeclaration {
    pub import_list: Vec<LibraryImport>,
    pub top_level_declaration_list: Vec<Box<Node>>,
}

#[derive(Debug)]
pub struct LibraryImport {
    pub uri: Span,
    pub identifier: Option<Box<Node>>,
}

#[derive(Debug)]
pub struct FunctionParameter {
    pub identifier: Box<Node>,
}

#[derive(Debug)]
pub struct StatementLabel {
    pub identifier: Box<Node>,
}
