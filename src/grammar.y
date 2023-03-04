%start LibraryDeclaration
%expect 5
%%
// シフト還元競合:
// IfStatement (if(Expression)Statementとif(Expression)Statement else Statement)
// Selector (.Identifierと.identifier())
// FinalConstVarOrTypeのType周りに3つ

//----------------------------------------------------------------------
//-----------------------------Variables--------------------------------
//----------------------------------------------------------------------
InitializedVariableDeclaration -> Result<Vec<VariableDeclaration>, ()>:
      DeclaredIdentifier { 
        Ok(vec![VariableDeclaration { identifier: Box::new($1?), expr: None }]) 
    }
    | DeclaredIdentifier "=" Expression { 
        Ok(vec![VariableDeclaration { identifier: Box::new($1?), expr: Some(Box::new($3?)) }]) 
    }
    | InitializedVariableDeclaration "," InitializedIdentifier {
        flatten($1, $3?)
    }
    ;

InitializedIdentifier -> Result<VariableDeclaration, ()>:
      Identifier {
        Ok(VariableDeclaration { identifier: Box::new($1?), expr: None })
    }
    | Identifier "=" Expression {
        Ok(VariableDeclaration { identifier: Box::new($1?), expr: Some(Box::new($3?)) })
    }
    ;

InitializedIdentifierList -> Result<Vec<VariableDeclaration>, ()>:
      InitializedIdentifier { Ok(vec![$1?]) }
    | InitializedIdentifierList "," InitializedIdentifier { flatten($1, $3?) }
    ;


//----------------------------------------------------------------------
//-----------------------------Functions--------------------------------
//----------------------------------------------------------------------
FunctionSignature -> Result<FunctionSignature, ()>:
      Identifier FormalParameterList {
        Ok(FunctionSignature { return_type: None, name: Box::new($1?), param: $2? })
    }
    | Type Identifier FormalParameterList {
        Ok(FunctionSignature { return_type: Some($1?), name: Box::new($2?), param: $3? })
    }
    ;

FunctionBody -> Result<Node, ()>:
      "=>" Expression ";" { Ok(Node::ExpressionStatement { span: $span, expr: Box::new($2?) }) }
    | BlockStatement { $1 }
    ;

BlockStatement -> Result<Node, ()>:
      "{" Statements "}" { Ok(Node::BlockStatement { span: $span, children: $2? }) }
    ;

FormalParameterList -> Result<FunctionParamSignature, ()>:
      "(" ")" {
        Ok(FunctionParamSignature { normal_list: vec![], option_list: vec![], named_list: vec![] })
    }
    | "(" NormalFormalParameterList CommaOpt ")" {
        Ok(FunctionParamSignature { normal_list: $2?, option_list: vec![], named_list: vec![] })
    }
    | "(" NormalFormalParameterList "," OptionalOrNamedFormalParameterList ")" {
        let param = $4?;
        if param.1 {
            Ok(FunctionParamSignature { normal_list: $2?, option_list: param.0, named_list: vec![] })
        }
        else {
            Ok(FunctionParamSignature { normal_list: $2?, option_list: vec![], named_list: param.0 })
        }
    }
    | "(" OptionalOrNamedFormalParameterList ")" {
        let param = $2?;
        if param.1 {
            Ok(FunctionParamSignature { normal_list: vec![], option_list: param.0, named_list: vec![] })
        }
        else {
            Ok(FunctionParamSignature { normal_list: vec![], option_list: vec![], named_list: param.0 })
        }
    }
    ;

NormalFormalParameterList -> Result<Vec<FunctionParameter>, ()>:
      NormalFormalParameter { Ok(vec![FunctionParameter { identifier: Box::new($1?), expr: None }]) }
    | NormalFormalParameterList "," NormalFormalParameter { flatten($1, FunctionParameter { identifier: Box::new($3?), expr: None }) }
    ;

OptionalOrNamedFormalParameterList -> Result<(Vec<FunctionParameter>, bool), ()>:
      OptionalPositionalFormalParameterList {
        Ok(($1?, true))
    }
    | NamedFormalParameterList {
        Ok(($1?, false))
    }
    ;

OptionalPositionalFormalParameterList -> Result<Vec<FunctionParameter>, ()>:
    "[" OptionalPositionalFormalParameterListInternal CommaOpt "]" { $2 }
    ;

OptionalPositionalFormalParameterListInternal -> Result<Vec<FunctionParameter>, ()>:
      DefaultFormalParameter { Ok(vec![$1?]) }
    | OptionalPositionalFormalParameterListInternal "," DefaultFormalParameter {
        flatten($1, $3?)
    }
    ;

NamedFormalParameterList -> Result<Vec<FunctionParameter>, ()>:
    "{" NamedFormalParameterListInternal CommaOpt "}" { Ok($2?) }
    ;

NamedFormalParameterListInternal -> Result<Vec<FunctionParameter>, ()>:
      DefaultNamedParameter { Ok(vec![$1?]) }
    | NamedFormalParameterListInternal "," DefaultNamedParameter {
        flatten($1, $3?)
    }
    ;

NormalFormalParameter -> Result<Node, ()>:
      DeclaredIdentifier { $1 }
    | Identifier { $1 }
    ;

DefaultFormalParameter -> Result<FunctionParameter, ()>:
      NormalFormalParameter {
        Ok(FunctionParameter { identifier: Box::new($1?), expr: None })
    }
    | NormalFormalParameter "=" Expression {
        Ok(FunctionParameter { identifier: Box::new($1?), expr: Some(Box::new($3?)) })
    }
    ;

DefaultNamedParameter -> Result<FunctionParameter, ()>:
      DeclaredIdentifier {
        Ok(FunctionParameter { identifier: Box::new($1?), expr: None })
    }
    | Identifier {
        Ok(FunctionParameter { identifier: Box::new($1?), expr: None })
    }
    | DeclaredIdentifier "=" Expression {
        Ok(FunctionParameter { identifier: Box::new($1?), expr: Some(Box::new($3?)) })
    }
    | Identifier ":" Expression {
        Ok(FunctionParameter { identifier: Box::new($1?), expr: Some(Box::new($3?)) })
    }
    | "required" DeclaredIdentifier {
        Ok(FunctionParameter { identifier: Box::new($2?), expr: None })
    }
    | "required" Identifier {
        Ok(FunctionParameter { identifier: Box::new($2?), expr: None })
    }
    | "required" DeclaredIdentifier "=" Expression {
        Ok(FunctionParameter { identifier: Box::new($2?), expr: Some(Box::new($4?)) })
    }
    | "required" Identifier ":" Expression {
        Ok(FunctionParameter { identifier: Box::new($2?), expr: Some(Box::new($4?)) })
    }
    ;

DeclaredIdentifier -> Result<Node, ()>:
      "var" Identifier { $2 }
    | Type Identifier { $2 }
    | "late" "var" Identifier { $3 }
    | "late" Type Identifier { $3 }
    | "const" Identifier { $2 }
    | "const" Type Identifier { $3 }
    | "final" Identifier { $2 }
    | "final" Type Identifier { $3 }
    | "late" "final" Identifier { $3 }
    | "late" "final" Type Identifier { $4 }
    ;

//----------------------------------------------------------------------
//-----------------------------Expressions--------------------------------
//----------------------------------------------------------------------
Expression -> Result<Node, ()>:
      SelectorExpression AssignmentOperator Expression {
        Ok(Node::AssignmentExpression { span: $span, operator: $2?, left: Box::new($1?), right: Box::new($3?) })
    }
    | ThrowExpression { $1 }
    | ConditionalExpression { $1 }
    ;

ExpressionOpt -> Result<Option<Box<Node>>, ()>:
      %empty { Ok(None) }
    | Expression { Ok(Some(Box::new($1?))) }
    ;

ExpressionNotBrace -> Result<Node, ()>:
      SelectorExpressionNotBrace AssignmentOperator Expression {
        Ok(Node::AssignmentExpression { span: $span, operator: $2?, left: Box::new($1?), right: Box::new($3?) })
    }
    | ThrowExpression { $1 }
    | ConditionalExpressionNotBrace { $1 }
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

PrimaryNotBrace -> Result<Node, ()>:
      '(' Expression ')' { $2 }
    | LiteralNotBrace { $1 }
    | Identifier { $1 }
    ;

Identifier -> Result<Node, ()>:
    'IDENTIFIER' { Ok(Node::Identifier { span: $span }) }
    ;

Literal -> Result<Node, ()>:
      'NUMBER' { Ok(Node::NumericLiteral { span: $span }) }
    | StringLiteralList { Ok(Node::StringLiteral { span: $span, literal_list: $1? }) }
    | ListLiteral { $1 }
    | SetOrMapLiteral { $1 }
    | 'BOOLEAN' { Ok(Node::BooleanLiteral { span: $span }) }
    | 'NULL' { Ok(Node::NullLiteral { span: $span }) }
    ;

LiteralNotBrace -> Result<Node, ()>:
      'NUMBER' { Ok(Node::NumericLiteral { span: $span }) }
    | StringLiteralList { Ok(Node::StringLiteral { span: $span, literal_list: $1? }) }
    | ListLiteral { $1 }
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

ListLiteral -> Result<Node, ()>:
      "[" "]" {
        Ok(Node::ListLiteral { span: $span, element_list: vec![] })
    }
    | "const" "[" "]" {
        Ok(Node::ListLiteral { span: $span, element_list: vec![] })
    }
    | "[" ElementList CommaOpt "]" {
        Ok(Node::ListLiteral { span: $span, element_list: $2? })
    }
    | "const" "[" ElementList CommaOpt "]" {
        Ok(Node::ListLiteral { span: $span, element_list: $3? })
    }
    | TypeArguments "[" "]" {
        Ok(Node::ListLiteral { span: $span, element_list: vec![] })
    }
    | "const" TypeArguments "[" "]" {
        Ok(Node::ListLiteral { span: $span, element_list: vec![] })
    }
    | TypeArguments "[" ElementList CommaOpt "]" {
        Ok(Node::ListLiteral { span: $span, element_list: $3? })
    }
    | "const" TypeArguments "[" ElementList CommaOpt "]" {
        Ok(Node::ListLiteral { span: $span, element_list: $4? })
    }
    ;

SetOrMapLiteral -> Result<Node, ()>:
      "{" "}" {
        Ok(Node::SetOrMapLiteral { span: $span, element_list: vec![] })
    }
    | "const" "{" "}" {
        Ok(Node::SetOrMapLiteral { span: $span, element_list: vec![] })
    }
    | "{" ElementList CommaOpt "}" {
        Ok(Node::SetOrMapLiteral { span: $span, element_list: $2? })
    }
    | "const" "{" ElementList CommaOpt "}" {
        Ok(Node::SetOrMapLiteral { span: $span, element_list: $3? })
    }
    | TypeArguments "{" "}" {
        Ok(Node::SetOrMapLiteral { span: $span, element_list: vec![] })
    }
    | "const" TypeArguments "{" "}" {
        Ok(Node::SetOrMapLiteral { span: $span, element_list: vec![] })
    }
    | TypeArguments "{" ElementList CommaOpt "}" {
        Ok(Node::SetOrMapLiteral { span: $span, element_list: $3? })
    }
    | "const" TypeArguments "{" ElementList CommaOpt "}" {
        Ok(Node::SetOrMapLiteral { span: $span, element_list: $4? })
    }
    ;

ElementList -> Result<Vec<CollectionElement>, ()>:
      Element { Ok(vec![$1?]) }
    | ElementList "," Element { flatten($1, $3?) }
    ;

Element -> Result<CollectionElement, ()>:
      ExpressionElement { $1 }
    | MapElement { $1 }
    ;

ExpressionElement -> Result<CollectionElement, ()>:
    Expression {
        Ok(CollectionElement::ExpressionElement { expr: Box::new($1?) })
    }
    ;

MapElement -> Result<CollectionElement, ()>:
    Expression ":" Expression {
        Ok(CollectionElement::MapElement { key_expr: Box::new($1?), value_expr: Box::new($3?) })
    }
    ;

ThrowExpression -> Result<Node, ()>:
    "throw" Expression {
        Ok(Node::ThrowExpression { span: $span, expr: Box::new($2?) })
    }
    ;

ConditionalExpression -> Result<Node, ()>:
      IfNullExpression { $1 }
    | IfNullExpression "?" Expression ":" Expression {
        Ok(Node::ConditionalExpression { span: $span, condition: Box::new($1?), if_true_expr: Box::new($3?), if_false_expr: Box::new($5?) })
    }
    ;

ConditionalExpressionNotBrace -> Result<Node, ()>:
      IfNullExpressionNotBrace { $1 }
    | IfNullExpressionNotBrace "?" Expression ":" Expression {
        Ok(Node::ConditionalExpression { span: $span, condition: Box::new($1?), if_true_expr: Box::new($3?), if_false_expr: Box::new($5?) })
    }
    ;

IfNullExpression -> Result<Node, ()>:
      LogicalOrExpression { $1 }
    | IfNullExpression "??" LogicalOrExpression{
        Ok(Node::BinaryExpression { span: $span, operator: "??", left: Box::new($1?), right: Box::new($3?) })
    }
    ;

IfNullExpressionNotBrace -> Result<Node, ()>:
      LogicalOrExpressionNotBrace { $1 }
    | IfNullExpressionNotBrace "??" LogicalOrExpression{
        Ok(Node::BinaryExpression { span: $span, operator: "??", left: Box::new($1?), right: Box::new($3?) })
    }
    ;

LogicalOrExpression -> Result<Node, ()>:
      LogicalAndExpression { $1 }
    | LogicalOrExpression "||" LogicalAndExpression{
        Ok(Node::BinaryExpression { span: $span, operator: "||", left: Box::new($1?), right: Box::new($3?) })
    }
    ;

LogicalOrExpressionNotBrace -> Result<Node, ()>:
      LogicalAndExpressionNotBrace { $1 }
    | LogicalOrExpressionNotBrace "||" LogicalAndExpression{
        Ok(Node::BinaryExpression { span: $span, operator: "||", left: Box::new($1?), right: Box::new($3?) })
    }
    ;

LogicalAndExpression -> Result<Node, ()>:
      EqualityExpression { $1 }
    | LogicalAndExpression "&&" EqualityExpression{
        Ok(Node::BinaryExpression { span: $span, operator: "&&", left: Box::new($1?), right: Box::new($3?) })
    }
    ;

LogicalAndExpressionNotBrace -> Result<Node, ()>:
      EqualityExpressionNotBrace { $1 }
    | LogicalAndExpressionNotBrace "&&" EqualityExpression{
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

EqualityExpressionNotBrace -> Result<Node, ()>:
      RelationalExpressionNotBrace "==" RelationalExpression {
        Ok(Node::BinaryExpression { span: $span, operator: "==", left: Box::new($1?), right: Box::new($3?) })
    }
    | RelationalExpressionNotBrace "!=" RelationalExpression {
        Ok(Node::BinaryExpression { span: $span, operator: "!=", left: Box::new($1?), right: Box::new($3?) })
    }
    | RelationalExpressionNotBrace { $1 }
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
    | BitwiseOrExpression TypeTest {
        Ok(Node::TypeTestExpression { span: $span, child: Box::new($1?), type_test: $2? })
    }
    | BitwiseOrExpression TypeCast {
        Ok(Node::TypeCastExpression { span: $span, child: Box::new($1?), type_cast: $2? })
    }
    | BitwiseOrExpression { $1 }
    ;

RelationalExpressionNotBrace -> Result<Node, ()>:
      BitwiseOrExpressionNotBrace ">=" BitwiseOrExpression {
        Ok(Node::BinaryExpression { span: $span, operator: ">=", left: Box::new($1?), right: Box::new($3?) })
    }
    | BitwiseOrExpressionNotBrace ">" BitwiseOrExpression {
        Ok(Node::BinaryExpression { span: $span, operator: ">", left: Box::new($1?), right: Box::new($3?) })
    }
    | BitwiseOrExpressionNotBrace "<=" BitwiseOrExpression {
        Ok(Node::BinaryExpression { span: $span, operator: "<=", left: Box::new($1?), right: Box::new($3?) })
    }
    | BitwiseOrExpressionNotBrace "<" BitwiseOrExpression {
        Ok(Node::BinaryExpression { span: $span, operator: "<", left: Box::new($1?), right: Box::new($3?) })
    }
    | BitwiseOrExpressionNotBrace TypeTest {
        Ok(Node::TypeTestExpression { span: $span, child: Box::new($1?), type_test: $2? })
    }
    | BitwiseOrExpressionNotBrace TypeCast {
        Ok(Node::TypeCastExpression { span: $span, child: Box::new($1?), type_cast: $2? })
    }
    | BitwiseOrExpressionNotBrace { $1 }
    ;

BitwiseOrExpression -> Result<Node, ()>:
      BitwiseOrExpression "|" BitwiseXorExpression {
        Ok(Node::BinaryExpression { span: $span, operator: "|", left: Box::new($1?), right: Box::new($3?) })
    }
    | BitwiseXorExpression { $1 }
    ;

BitwiseOrExpressionNotBrace -> Result<Node, ()>:
      BitwiseOrExpressionNotBrace "|" BitwiseXorExpression {
        Ok(Node::BinaryExpression { span: $span, operator: "|", left: Box::new($1?), right: Box::new($3?) })
    }
    | BitwiseXorExpressionNotBrace { $1 }
    ;

BitwiseXorExpression -> Result<Node, ()>:
      BitwiseXorExpression "^" BitwiseAndExpression {
        Ok(Node::BinaryExpression { span: $span, operator: "^", left: Box::new($1?), right: Box::new($3?) })
    }
    | BitwiseAndExpression { $1 }
    ;

BitwiseXorExpressionNotBrace -> Result<Node, ()>:
      BitwiseXorExpressionNotBrace "^" BitwiseAndExpression {
        Ok(Node::BinaryExpression { span: $span, operator: "^", left: Box::new($1?), right: Box::new($3?) })
    }
    | BitwiseAndExpressionNotBrace { $1 }
    ;

BitwiseAndExpression -> Result<Node, ()>:
      BitwiseAndExpression "&" ShiftExpression {
        Ok(Node::BinaryExpression { span: $span, operator: "&", left: Box::new($1?), right: Box::new($3?) })
    }
    | ShiftExpression { $1 }
    ;

BitwiseAndExpressionNotBrace -> Result<Node, ()>:
      BitwiseAndExpressionNotBrace "&" ShiftExpression {
        Ok(Node::BinaryExpression { span: $span, operator: "&", left: Box::new($1?), right: Box::new($3?) })
    }
    | ShiftExpressionNotBrace { $1 }
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

ShiftExpressionNotBrace -> Result<Node, ()>:
      ShiftExpressionNotBrace "<<" AdditiveExpression {
        Ok(Node::BinaryExpression { span: $span, operator: "<<", left: Box::new($1?), right: Box::new($3?) })
    }
    | ShiftExpressionNotBrace ">>" AdditiveExpression {
        Ok(Node::BinaryExpression { span: $span, operator: ">>", left: Box::new($1?), right: Box::new($3?) })
    }
    | AdditiveExpressionNotBrace { $1 }
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

AdditiveExpressionNotBrace -> Result<Node, ()>:
      AdditiveExpressionNotBrace '+' MultiplicativeExpression { 
        Ok(Node::BinaryExpression { span: $span, operator: "+", left: Box::new($1?), right: Box::new($3?) })
    }
    | AdditiveExpressionNotBrace '-' MultiplicativeExpression { 
        Ok(Node::BinaryExpression { span: $span, operator: "-", left: Box::new($1?), right: Box::new($3?) })
    }
    | MultiplicativeExpressionNotBrace { $1 }
    ;

MultiplicativeExpression -> Result<Node, ()>:
      MultiplicativeExpression '*' Primary { 
        Ok(Node::BinaryExpression { span: $span, operator: "*", left: Box::new($1?), right: Box::new($3?) })
    }
    | MultiplicativeExpression '/' Primary { 
        Ok(Node::BinaryExpression { span: $span, operator: "/", left: Box::new($1?), right: Box::new($3?) })
    }
    | MultiplicativeExpression '%' Primary { 
        Ok(Node::BinaryExpression { span: $span, operator: "%", left: Box::new($1?), right: Box::new($3?) })
    }
    | MultiplicativeExpression '~/' Primary { 
        Ok(Node::BinaryExpression { span: $span, operator: "~/", left: Box::new($1?), right: Box::new($3?) })
    }
    | UnaryExpression { $1 }
    ;

MultiplicativeExpressionNotBrace -> Result<Node, ()>:
      MultiplicativeExpressionNotBrace '*' Primary { 
        Ok(Node::BinaryExpression { span: $span, operator: "*", left: Box::new($1?), right: Box::new($3?) })
    }
    | MultiplicativeExpressionNotBrace '/' Primary { 
        Ok(Node::BinaryExpression { span: $span, operator: "/", left: Box::new($1?), right: Box::new($3?) })
    }
    | MultiplicativeExpressionNotBrace '%' Primary { 
        Ok(Node::BinaryExpression { span: $span, operator: "%", left: Box::new($1?), right: Box::new($3?) })
    }
    | MultiplicativeExpressionNotBrace '~/' Primary { 
        Ok(Node::BinaryExpression { span: $span, operator: "~/", left: Box::new($1?), right: Box::new($3?) })
    }
    | UnaryExpressionNotBrace { $1 }
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
    | "++" SelectorExpression {
        Ok(Node::UpdateExpression { span: $span, operator: "++", is_prefix: true, child: Box::new($2?) })
    }
    | "--" SelectorExpression {
        Ok(Node::UpdateExpression { span: $span, operator: "--", is_prefix: true, child: Box::new($2?) })
    }
    | PostfixExpression { $1 }
    ;

UnaryExpressionNotBrace -> Result<Node, ()>:
      "-" UnaryExpression {
        Ok(Node::UnaryOpExpression { span: $span, operator: "-", child: Box::new($2?) })
    }
    | "!" UnaryExpression {
        Ok(Node::UnaryOpExpression { span: $span, operator: "!", child: Box::new($2?) })
    }
    | "~" UnaryExpression {
        Ok(Node::UnaryOpExpression { span: $span, operator: "~", child: Box::new($2?) })
    }
    | "++" SelectorExpression {
        Ok(Node::UpdateExpression { span: $span, operator: "++", is_prefix: true, child: Box::new($2?) })
    }
    | "--" SelectorExpression {
        Ok(Node::UpdateExpression { span: $span, operator: "--", is_prefix: true, child: Box::new($2?) })
    }
    | PostfixExpressionNotBrace { $1 }
    ;

PostfixExpression -> Result<Node, ()>:
      SelectorExpression { $1 }
    | Primary "++" {
        Ok(Node::UpdateExpression { span: $span, operator: "++", is_prefix: false, child: Box::new($1?) })
    }
    | Primary "--" {
        Ok(Node::UpdateExpression { span: $span, operator: "--", is_prefix: false, child: Box::new($1?) })
    }
    ;

PostfixExpressionNotBrace -> Result<Node, ()>:
      SelectorExpressionNotBrace { $1 }
    | PrimaryNotBrace "++" {
        Ok(Node::UpdateExpression { span: $span, operator: "++", is_prefix: false, child: Box::new($1?) })
    }
    | PrimaryNotBrace "--" {
        Ok(Node::UpdateExpression { span: $span, operator: "--", is_prefix: false, child: Box::new($1?) })
    }
    ;

SelectorExpression -> Result<Node, ()>:
      Primary { $1 }
    | SelectorExpression Selector {
        Ok(Node::SelectorExpression { span: $span, child: Box::new($1?), selector: $2? })
    }
    ;

SelectorExpressionNotBrace -> Result<Node, ()>:
      PrimaryNotBrace { $1 }
    | SelectorExpressionNotBrace Selector {
        Ok(Node::SelectorExpression { span: $span, child: Box::new($1?), selector: $2? })
    }
    ;

Selector -> Result<Selector, ()>:
      Arguments { 
        Ok(Selector::Args { span: $span, args: Box::new($1?) })
    }
    | "." Identifier { 
        Ok(Selector::Attr { span: $span, identifier: Box::new($2?) }) 
    }
    | "." Identifier Arguments {
        Ok(Selector::Method { span: $span, identifier: Box::new($2?), arguments: Box::new($3?) })
    }
    | "[" Expression "]" {
        Ok(Selector::Index { span: $span, expr: Box::new($2?) })
    }
    ;

//ArgumentPart -> Result<Node, ()>:
//      Arguments { $1 }
//    | TypeArguments Arguments { $2 }
//    ;

Arguments -> Result<Node, ()>:
      "(" ")" { Ok(Node::Arguments { span: $span, children: vec![] }) }
    | "(" ArgumentList CommaOpt ")" { Ok(Node::Arguments { span: $span, children: $2? }) }
    ;

ArgumentList -> Result<Vec<CallParameter>, ()>:
      NamedArgument { Ok(vec![$1?]) }
    | NormalArgument { Ok(vec![$1?]) }
    | ArgumentList "," NamedArgument { flatten($1, $3?) }
    | ArgumentList "," NormalArgument { flatten($1, $3?) }
    ;

NamedArgument -> Result<CallParameter, ()>:
    Label Expression { Ok(CallParameter { identifier: Some(Box::new($1?)), expr: Box::new($2?) }) }
    ;

NormalArgument -> Result<CallParameter, ()>:
      Expression { Ok(CallParameter { identifier: None, expr: Box::new($1?) }) }
    ;




//LateOpt -> Result<(), ()>:
//      %empty { Ok(()) }
//    | "late" { Ok(()) }
//    ;
//
//TypeOpt -> Result<(), ()>:
//      %empty { Ok(()) }
//    | Type { Ok(()) }
//    ;

// TypeParameter ::=
//     | Identifier
//     | Identifier extends TypeNotVoid
// 
// TypeParameters ::= < TypeParametersInternal >
// TypeParametersInternal ::=
//     | TypeParameter
//     | TypeParametersInternal , TypeParameter

TypeTest -> Result<TypeTest, ()>:
      "is" TypeNotVoid { Ok(TypeTest { dart_type: $2?, check_matching: true }) }
    | "is" "!" TypeNotVoid { Ok(TypeTest { dart_type: $3?, check_matching: false }) }
    ;

TypeCast -> Result<DartType, ()>:
    "as" TypeNotVoid { $2 }
    ;



//----------------------------------------------------------------------
//----------------------------Statements--------------------------------
//----------------------------------------------------------------------
Statements -> Result<Vec<Box<Node>>, ()>:
      %empty { Ok(vec![]) }
    | Statements Statement { flatten($1, Box::new($2?)) }
    ;

Statement -> Result<Node, ()>:
      NonLabeledStatement { $1 }
    | Label NonLabeledStatement {
        Ok(Node::LabeledStatement { span: $span, label: Box::new($1?), stmt: Box::new($2?) })
    }
    ;

NonLabeledStatement -> Result<Node, ()>:
      BlockStatement { $1 }
    | LocalVariableDeclaration { $1 }
    | IfStatement { $1 }
    | RethrowStatement { $1 }
    | TryStatement { $1 }
    | ForStatement { $1 }
    | WhileStatement { $1 }
    | DoStatement { $1 }
    | SwitchStatement { $1 }
    | BreakStatement { $1 }
    | ContinueStatement { $1 }
    | ReturnStatement { $1 }
    | ExpressionStatement { $1 }
    | ";" { Ok(Node::EmptyStatement { span: $span }) }
    ;

ExpressionStatement -> Result<Node, ()>:
    ExpressionNotBrace ";" { Ok(Node::ExpressionStatement { span: $span, expr: Box::new($1?) }) }
    ;

LocalVariableDeclaration -> Result<Node, ()>:
    InitializedVariableDeclaration ";" {
        Ok(Node::VariableDeclarationList { span: $span, decl_list: $1? })
    }
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

SwitchStatement -> Result<Node, ()>:
    "switch" "(" Expression ")" "{" SwitchCaseList DefaultCaseOpt "}" {
        Ok(Node::SwitchStatement { span: $span, expr: Box::new($3?), case_list: $6?, default_case: $7? })
    }
    ;

SwitchCaseList -> Result<Vec<SwitchCase>, ()>:
      %empty { Ok(vec![]) }
    | SwitchCaseList SwitchCase { flatten($1, $2?) }
    ;

SwitchCase -> Result<SwitchCase, ()>:
      "case" Expression ":" Statements {
        Ok(SwitchCase { label_list: vec![], expr: Box::new($2?), stmt_list: $4? })
    }
    ;

DefaultCase -> Result<DefaultCase, ()>:
      "default" ":" Statements {
        Ok(DefaultCase { label_list: vec![], stmt_list: $3? })
    }
    ;

DefaultCaseOpt -> Result<Option<DefaultCase>, ()>:
      %empty { Ok(None) }
    | DefaultCase { Ok(Some($1?)) }
    ;

RethrowStatement -> Result<Node, ()>:
    "rethrow" ";" {
        Ok(Node::RethrowStatement { span: $span })
    }
    ;

TryStatement -> Result<Node, ()>:
      "try" BlockStatement FinallyPart {
        Ok(Node::TryFinallyStatement { span: $span, block_try: Box::new($2?), block_finally: Box::new($3?) })
    }
    | "try" BlockStatement OnPartList {
        Ok(Node::TryOnStatement { span: $span, block_try: Box::new($2?), on_part_list: $3? })
    }
    | "try" BlockStatement OnPartList FinallyPart {
        Ok(Node::TryFinallyStatement { 
            span: $span, 
            block_try: Box::new(Node::TryOnStatement {
                span: $span,
                block_try: Box::new($2?),
                on_part_list: $3?,
            }),
            block_finally: Box::new($4?), 
        })
    }
    ;

OnPartList -> Result<Vec<TryOnPart>, ()>:
      OnPart { Ok(vec![$1?]) }
    | OnPartList OnPart { flatten($1, $2?) }
    ;

OnPart -> Result<TryOnPart, ()>:
      CatchPart BlockStatement {
        Ok(TryOnPart { catch_part: Some($1?), exc_type: None, block: Box::new($2?) })
    }
    | "on" TypeNotVoid BlockStatement {
        Ok(TryOnPart { catch_part: None, exc_type: Some($2?), block: Box::new($3?) })
    }
    | "on" TypeNotVoid CatchPart BlockStatement {
        Ok(TryOnPart { catch_part: Some($3?), exc_type: Some($2?), block: Box::new($4?) })
    }
    ;

CatchPart -> Result<TryCatchPart, ()>:
      "catch" "(" Identifier ")" {
        Ok(TryCatchPart { id_error: Box::new($3?), id_trace: None })
    }
    | "catch" "(" Identifier "," Identifier ")" {
        Ok(TryCatchPart { id_error: Box::new($3?), id_trace: Some(Box::new($5?)) })
    }
    ;

FinallyPart -> Result<Node, ()>:
    "finally" BlockStatement { $2 }
    ;

ReturnStatement -> Result<Node, ()>:
      "return" ";" { Ok(Node::ReturnStatement { span: $span, value: None }) }
    | "return" Expression ";" {
        Ok(Node::ReturnStatement { span: $span, value: Some(Box::new($2?)) })
    }
    ;

Label -> Result<Node, ()>:
    Identifier ":" { $1 }
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




//----------------------------------------------------------------------
//-----------------------Libraries and Scripts--------------------------
//----------------------------------------------------------------------
TopLevelDeclaration -> Result<Node, ()>:
      TopFunctionDeclaration { $1 }
    | TopVariableDeclaration { $1 }
    ;

LibraryDeclaration -> Result<LibraryDeclaration, ()>:
    LibraryImportList TopLevelDeclarationList { 
        Ok(LibraryDeclaration { import_list: $1?, top_level_declaration_list: $2? })
    }
    ;

TopLevelDeclarationList -> Result<Vec<Box<Node>>, ()>:
      %empty { Ok(vec![]) }
    | TopLevelDeclarationList TopLevelDeclaration { flatten($1, Box::new($2?)) }
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

TopFunctionDeclaration -> Result<Node, ()>:
    FunctionSignature FunctionBody {
        Ok(Node::FunctionDeclaration { span: $span, signature: $1?, body: Box::new($2?) })
    }
    ;

TopVariableDeclaration -> Result<Node, ()>:
      "var" InitializedIdentifierList ";" { 
        Ok(Node::VariableDeclarationList { span: $span, decl_list: $2? }) 
    }
    | Type InitializedIdentifierList ";" { 
        Ok(Node::VariableDeclarationList { span: $span, decl_list: $2? }) 
    }
    | "late" "var" InitializedIdentifierList ";" { 
        Ok(Node::VariableDeclarationList { span: $span, decl_list: $3? }) 
    }
    | "late" Type InitializedIdentifierList ";" { 
        Ok(Node::VariableDeclarationList { span: $span, decl_list: $3? }) 
    }
    | "late" "final" InitializedIdentifierList ";" { 
        Ok(Node::VariableDeclarationList { span: $span, decl_list: $3? }) 
    }
    | "late" "final" Type InitializedIdentifierList ";" { 
        Ok(Node::VariableDeclarationList { span: $span, decl_list: $4? }) 
    }
    ;




//----------------------------------------------------------------------
//--------------------------Static Types--------------------------------
//----------------------------------------------------------------------
Type -> Result<DartType, ()>:
    TypeNotFunction { $1 }
    ;

TypeNotVoid -> Result<DartType, ()>:
    TypeNotVoidNotFunction { $1 }
    ;

TypeNotFunction -> Result<DartType, ()>:
      "void" { Ok(DartType::Void { span: $span }) }
    | TypeNotVoidNotFunction { $1 }
    ;

TypeNotVoidNotFunction -> Result<DartType, ()>:
      TypeName { 
        Ok(DartType::Named { span: $span, type_name: $1?, type_arguments: vec![], is_nullable: false }) }
    ;

TypeName -> Result<DartTypeName, ()>:
      Identifier {
        Ok(DartTypeName { identifier: Box::new($1?), module: None })
    }
    | Identifier "." Identifier {
        Ok(DartTypeName { identifier: Box::new($3?), module: Some(Box::new($1?)) })
    }
    ;

TypeArguments -> Result<Vec<DartType>, ()>:
    "<" TypeList ">" { $2 }
    ;
TypeList -> Result<Vec<DartType>, ()>:
      Type { Ok(vec![$1?]) }
    | TypeList "," Type { flatten($1, $3?) }
    ;

// FunctionType ::=
//     | FunctionTypeTails
//     | TypeNotFunction FunctionTypeTails
// FunctionTypeTails ::=
//     | FunctionTypeTail
//     | FunctionTypeTail FunctionTypeTails
//     | FunctionTypeTail ? FunctionTypeTails
// FunctionTypeTail ::=
//     | Function ParameterTypeList
//     | Function TypeParameters ParameterTypeList
// ParameterTypeList ::=
//     | ( )
//     | ( NormalParameterType , OptionalParameterTypes )
//     | ( NormalParameterTypes CommaOpt )
//     | ( OptionalParameterTypes )
// NormalParameterTypes ::=
//     | NormalParameterType
//     | NormalParameterTypes , NormalParameterType
// NormalParameterType ::=
//     | TypedIdentifier
//     | Type
// OptionalParameterTypes ::=
//     | OptionalPositionalParameterTypes
//     | NamedParameterTypes
// OptionalPositionalParameterTypes ::=
//     | [ NormalParameterTypes CommaOpt ]
// NamedParameterTypes ::=
//     | { NamedParameterTypesInternal CommaOpt }
// NamedParameterTypesInternal ::=
//     | NamedParameterType
//     | NamedParameterTypesInternal , NamedParameterType
// NamedParameterType ::=
//     | TypedIdentifier
//     | required TypedIdentifier
// TypedIdentifier ::=
//     | Type Identifier



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
    UpdateExpression {
        span: Span,
        operator: &'static str,
        is_prefix: bool,
        child: Box<Node>,
    },
    AssignmentExpression {
        span: Span,
        operator: &'static str,
        left: Box<Node>,
        right: Box<Node>,
    },
    TypeTestExpression {
        span: Span,
        child: Box<Node>,
        type_test: TypeTest,
    },
    TypeCastExpression {
        span: Span,
        child: Box<Node>,
        type_cast: DartType,
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
    ListLiteral {
        span: Span,
        element_list: Vec<CollectionElement>,
    },
    SetOrMapLiteral {
        span: Span,
        element_list: Vec<CollectionElement>,
    },
    Identifier {
        span: Span,
    },
    Arguments {
        span: Span,
        children: Vec<CallParameter>
    },
    SelectorExpression {
        span: Span,
        child: Box<Node>,
        selector: Selector,
    },
    ThrowExpression {
        span: Span,
        expr: Box<Node>,
    },

    LabeledStatement {
        span: Span,
        label: Box<Node>,
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
    VariableDeclarationList {
        span: Span,
        decl_list: Vec<VariableDeclaration>,
    },
    IfStatement {
        span: Span,
        condition: Box<Node>,
        if_true_stmt: Box<Node>,
        if_false_stmt: Option<Box<Node>>,
    },
    RethrowStatement {
        span: Span,
    },
    TryFinallyStatement {
        span: Span,
        block_try: Box<Node>,
        block_finally: Box<Node>,
    },
    TryOnStatement {
        span: Span,
        block_try: Box<Node>,
        on_part_list: Vec<TryOnPart>,
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
    SwitchStatement {
        span: Span,
        expr: Box<Node>,
        case_list: Vec<SwitchCase>,
        default_case: Option<DefaultCase>,
    },
    FunctionDeclaration {
        span: Span,
        signature: FunctionSignature,
        body: Box<Node>,
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
    pub expr: Option<Box<Node>>,
}

#[derive(Debug)]
pub struct SwitchCase {
    pub label_list: Vec<Box<Node>>,
    pub expr: Box<Node>,
    pub stmt_list: Vec<Box<Node>>,
}

#[derive(Debug)]
pub struct DefaultCase {
    pub label_list: Vec<Box<Node>>,
    pub stmt_list: Vec<Box<Node>>,
}

#[derive(Debug)]
pub struct TryOnPart {
    pub catch_part: Option<TryCatchPart>,
    pub exc_type: Option<DartType>,
    pub block: Box<Node>,
}

#[derive(Debug)]
pub struct TryCatchPart {
    pub id_error: Box<Node>,
    pub id_trace: Option<Box<Node>>,
}

#[derive(Debug)]
pub enum CollectionElement {
    ExpressionElement {
        expr: Box<Node>,
    },
    MapElement {
        key_expr: Box<Node>,
        value_expr: Box<Node>,
    },
}

#[derive(Debug)]
pub enum Selector {
    Index {
        span: Span,
        expr: Box<Node>,
    },
    Attr {
        span: Span,
        identifier: Box<Node>,
    },
    Method {
        span: Span,
        identifier: Box<Node>,
        arguments: Box<Node>,
    },
    Args {
        span: Span,
        args: Box<Node>,
    }
}

#[derive(Debug)]
pub struct TypeTest {
    pub dart_type: DartType,
    pub check_matching: bool,
}

#[derive(Debug)]
pub enum DartType {
    Named {
        span: Span,
        type_name: DartTypeName,
        type_arguments: Vec<DartType>,
        is_nullable: bool,
    },
    Void {
        span: Span,
    },
}

#[derive(Debug)]
pub struct DartTypeName {
    pub identifier: Box<Node>,
    pub module: Option<Box<Node>>,
}

#[derive(Debug)]
pub struct FunctionSignature {
    pub return_type: Option<DartType>,
    pub name: Box<Node>,
    pub param: FunctionParamSignature,
}

#[derive(Debug)]
pub struct VariableDeclaration {
    pub identifier: Box<Node>,
    pub expr: Option<Box<Node>>,
}

#[derive(Debug)]
pub struct FunctionParamSignature {
    pub normal_list: Vec<FunctionParameter>,
    pub option_list: Vec<FunctionParameter>,
    pub named_list: Vec<FunctionParameter>,
}

#[derive(Debug)]
pub struct CallParameter {
    pub identifier: Option<Box<Node>>,
    pub expr: Box<Node>,
}
