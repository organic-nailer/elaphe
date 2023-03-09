%start LibraryDeclaration
%expect 2
%%
// シフト還元競合:
// IfStatement (if(Expression)Statementとif(Expression)Statement else Statement)
// Selector (.Identifierと.identifier())
// FinalConstVarOrTypeのType周りに3つ

//----------------------------------------------------------------------
//-----------------------------Variables--------------------------------
//----------------------------------------------------------------------
InitializedVariableDeclaration -> Result<Vec<VariableDeclaration<'input>>, ()>:
      DeclaredIdentifier { 
        Ok(vec![VariableDeclaration { identifier: $1?, expr: None }]) 
    }
    | DeclaredIdentifier "=" Expression { 
        Ok(vec![VariableDeclaration { identifier: $1?, expr: Some(Box::new($3?)) }]) 
    }
    | InitializedVariableDeclaration "," InitializedIdentifier {
        flatten($1, $3?)
    }
    ;

InitializedIdentifier -> Result<VariableDeclaration<'input>, ()>:
      Identifier {
        Ok(VariableDeclaration { identifier: $1?, expr: None })
    }
    | Identifier "=" Expression {
        Ok(VariableDeclaration { identifier: $1?, expr: Some(Box::new($3?)) })
    }
    ;

InitializedIdentifierList -> Result<Vec<VariableDeclaration<'input>>, ()>:
      InitializedIdentifier { Ok(vec![$1?]) }
    | InitializedIdentifierList "," InitializedIdentifier { flatten($1, $3?) }
    ;


//----------------------------------------------------------------------
//-----------------------------Functions--------------------------------
//----------------------------------------------------------------------
FunctionSignature -> Result<FunctionSignature<'input>, ()>:
      Identifier FormalParameterList {
        Ok(FunctionSignature { return_type: None, name: $1?, param: $2? })
    }
    | Type Identifier FormalParameterList {
        Ok(FunctionSignature { return_type: Some($1?), name: $2?, param: $3? })
    }
    ;

FunctionBody -> Result<Node<'input>, ()>:
      "=>" Expression ";" { Ok(Node::ExpressionStatement { expr: Box::new($2?) }) }
    | BlockStatement { $1 }
    ;

BlockStatement -> Result<Node<'input>, ()>:
      "{" Statements "}" { Ok(Node::BlockStatement { children: $2? }) }
    ;

FormalParameterList -> Result<FunctionParamSignature<'input>, ()>:
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

NormalFormalParameterList -> Result<Vec<FunctionParameter<'input>>, ()>:
      NormalFormalParameter { Ok(vec![FunctionParameter { identifier: $1?, expr: None }]) }
    | NormalFormalParameterList "," NormalFormalParameter { flatten($1, FunctionParameter { identifier: $3?, expr: None }) }
    ;

OptionalOrNamedFormalParameterList -> Result<(Vec<FunctionParameter<'input>>, bool), ()>:
      OptionalPositionalFormalParameterList {
        Ok(($1?, true))
    }
    | NamedFormalParameterList {
        Ok(($1?, false))
    }
    ;

OptionalPositionalFormalParameterList -> Result<Vec<FunctionParameter<'input>>, ()>:
    "[" OptionalPositionalFormalParameterListInternal CommaOpt "]" { $2 }
    ;

OptionalPositionalFormalParameterListInternal -> Result<Vec<FunctionParameter<'input>>, ()>:
      DefaultFormalParameter { Ok(vec![$1?]) }
    | OptionalPositionalFormalParameterListInternal "," DefaultFormalParameter {
        flatten($1, $3?)
    }
    ;

NamedFormalParameterList -> Result<Vec<FunctionParameter<'input>>, ()>:
    "{" NamedFormalParameterListInternal CommaOpt "}" { Ok($2?) }
    ;

NamedFormalParameterListInternal -> Result<Vec<FunctionParameter<'input>>, ()>:
      DefaultNamedParameter { Ok(vec![$1?]) }
    | NamedFormalParameterListInternal "," DefaultNamedParameter {
        flatten($1, $3?)
    }
    ;

NormalFormalParameter -> Result<Identifier<'input>, ()>:
      DeclaredIdentifier { $1 }
    | Identifier { $1 }
    ;

DefaultFormalParameter -> Result<FunctionParameter<'input>, ()>:
      NormalFormalParameter {
        Ok(FunctionParameter { identifier: $1?, expr: None })
    }
    | NormalFormalParameter "=" Expression {
        Ok(FunctionParameter { identifier: $1?, expr: Some(Box::new($3?)) })
    }
    ;

DefaultNamedParameter -> Result<FunctionParameter<'input>, ()>:
      DeclaredIdentifier {
        Ok(FunctionParameter { identifier: $1?, expr: None })
    }
    | Identifier {
        Ok(FunctionParameter { identifier: $1?, expr: None })
    }
    | DeclaredIdentifier "=" Expression {
        Ok(FunctionParameter { identifier: $1?, expr: Some(Box::new($3?)) })
    }
    | Identifier ":" Expression {
        Ok(FunctionParameter { identifier: $1?, expr: Some(Box::new($3?)) })
    }
    | "required" DeclaredIdentifier {
        Ok(FunctionParameter { identifier: $2?, expr: None })
    }
    | "required" Identifier {
        Ok(FunctionParameter { identifier: $2?, expr: None })
    }
    | "required" DeclaredIdentifier "=" Expression {
        Ok(FunctionParameter { identifier: $2?, expr: Some(Box::new($4?)) })
    }
    | "required" Identifier ":" Expression {
        Ok(FunctionParameter { identifier: $2?, expr: Some(Box::new($4?)) })
    }
    ;

DeclaredIdentifier -> Result<Identifier<'input>, ()>:
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
//-----------------------------Classes----------------------------------
//----------------------------------------------------------------------
ClassDeclaration -> Result<Node<'input>, ()>:
      "class" Identifier "{" "}" {
        Ok(Node::ClassDeclaration { identifier: $2?, member_list: vec![] })
    }
    | "class" Identifier "{" ClassDeclarationInternal "}" {
        Ok(Node::ClassDeclaration { identifier: $2?, member_list: $4? })
    }
    ;

ClassDeclarationInternal -> Result<Vec<Member<'input>>, ()>:
      ClassMemberDeclaration { Ok(vec![$1?]) }
    | ClassDeclarationInternal ClassMemberDeclaration {
        flatten($1, $2?)
    }
    ;

ClassMemberDeclaration -> Result<Member<'input>, ()>:
      Declaration ";" { $1 }
    | MemberImpl { $1 }
    ;

MemberImpl -> Result<Member<'input>, ()>:
      FunctionSignature FunctionBody {
        Ok(Member::MethodImpl { signature: $1?, body: Box::new($2?) })
    }
//     | ConstructorSignature FunctionBody {
//         Ok(Member::ConstructorImpl { signature: $1?, body: Box::new($2?) })
//     }
    ;

Declaration -> Result<Member<'input>, ()>:
      "var" InitializedIdentifierList {
        Ok(Member::VariableDecl { decl_list: $2? }) 
    }
    | Type InitializedIdentifierList {
        Ok(Member::VariableDecl { decl_list: $2? }) 
    }
    | "late" "var" InitializedIdentifierList {
        Ok(Member::VariableDecl { decl_list: $3? }) 
    }
    | "late" Type InitializedIdentifierList {
        Ok(Member::VariableDecl { decl_list: $3? }) 
    }
    | "final" InitializedIdentifierList {
        Ok(Member::VariableDecl { decl_list: $2? }) 
    }
    | "final" Type InitializedIdentifierList {
        Ok(Member::VariableDecl { decl_list: $3? }) 
    }
    | "late" "final" InitializedIdentifierList {
        Ok(Member::VariableDecl { decl_list: $3? }) 
    }
    | "late" "final" Type InitializedIdentifierList {
        Ok(Member::VariableDecl { decl_list: $4? }) 
    }
    ;

// ConstructorSignature -> Result<ConstructorSignature<'input>, ()>:
//     ConstructorName FormalParameterList {
//         Ok(ConstructorSignature { name: $1?, param: $2? })
//     }
//     ;
// 
// ConstructorName -> Result<Option<Identifier<'input>>, ()>:
//       Identifier "." Identifier {
//         Ok(Some($3?))
//     }
//     ;

//----------------------------------------------------------------------
//-----------------------------Expressions--------------------------------
//----------------------------------------------------------------------
Expression -> Result<Node<'input>, ()>:
      SelectorExpression AssignmentOperator Expression {
        Ok(Node::AssignmentExpression { operator: $2?, left: Box::new($1?), right: Box::new($3?) })
    }
    | ThrowExpression { $1 }
    | ConditionalExpression { $1 }
    ;

ExpressionOpt -> Result<Option<Box<Node<'input>>>, ()>:
      %empty { Ok(None) }
    | Expression { Ok(Some(Box::new($1?))) }
    ;

ExpressionNotBrace -> Result<Node<'input>, ()>:
      SelectorExpressionNotBrace AssignmentOperator Expression {
        Ok(Node::AssignmentExpression { operator: $2?, left: Box::new($1?), right: Box::new($3?) })
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

ExpressionList -> Result<Vec<Box<Node<'input>>>, ()>:
      ExpressionList "," Expression { 
        flatten($1, Box::new($3?))
    }
    | Expression { Ok(vec![Box::new($1?)]) }
    ;

ExpressionListOpt -> Result<Option<Vec<Box<Node<'input>>>>, ()>:
      %empty { Ok(None) }
    | ExpressionList { Ok(Some($1?)) }
    ;

CommaOpt -> Result<(), ()>:
      %empty { Ok(()) }
    | "," { Ok(()) }
    ;

Primary -> Result<Node<'input>, ()>:
      '(' Expression ')' { $2 }
    | ThisExpression { $1 }
    | Literal { $1 }
    | Identifier { Ok(Node::IdentifierNode { identifier: $1? }) }
    ;

PrimaryNotBrace -> Result<Node<'input>, ()>:
      '(' Expression ')' { $2 }
    | ThisExpression { $1 }
    | LiteralNotBrace { $1 }
    | Identifier { Ok(Node::IdentifierNode { identifier: $1? }) }
    ;

Identifier -> Result<Identifier<'input>, ()>:
    'IDENTIFIER' { Ok(Identifier { value: $lexer.span_str($span) }) }
    ;

Literal -> Result<Node<'input>, ()>:
      'NUMBER' { Ok(Node::NumericLiteral { value: $lexer.span_str($span) }) }
    | StringLiteralList { Ok(Node::StringLiteral { literal_list: $1? }) }
    | ListLiteral { $1 }
    | SetOrMapLiteral { $1 }
    | 'BOOLEAN' { Ok(Node::BooleanLiteral { value: $lexer.span_str($span) }) }
    | 'NULL' { Ok(Node::NullLiteral) }
    ;

LiteralNotBrace -> Result<Node<'input>, ()>:
      'NUMBER' { Ok(Node::NumericLiteral { value: $lexer.span_str($span) }) }
    | StringLiteralList { Ok(Node::StringLiteral { literal_list: $1? }) }
    | ListLiteral { $1 }
    | 'BOOLEAN' { Ok(Node::BooleanLiteral { value: $lexer.span_str($span) }) }
    | 'NULL' { Ok(Node::NullLiteral) }
    ;

StringLiteralList -> Result<Vec<&'input str>, ()>:
      StringLiteralList "STRING" { 
        match $2 {
            Ok(v) => flatten($1, $lexer.span_str(v.span())),
            Err(_) => Err(())
        }
    }
    | "STRING" { 
        match $1 {
            Ok(v) => Ok(vec![$lexer.span_str(v.span())]),
            Err(_) => Err(())
        }
    }
    ;

ListLiteral -> Result<Node<'input>, ()>:
      "[" "]" {
        Ok(Node::ListLiteral { element_list: vec![] })
    }
    | "const" "[" "]" {
        Ok(Node::ListLiteral { element_list: vec![] })
    }
    | "[" ElementList CommaOpt "]" {
        Ok(Node::ListLiteral { element_list: $2? })
    }
    | "const" "[" ElementList CommaOpt "]" {
        Ok(Node::ListLiteral { element_list: $3? })
    }
    | TypeArguments "[" "]" {
        Ok(Node::ListLiteral { element_list: vec![] })
    }
    | "const" TypeArguments "[" "]" {
        Ok(Node::ListLiteral { element_list: vec![] })
    }
    | TypeArguments "[" ElementList CommaOpt "]" {
        Ok(Node::ListLiteral { element_list: $3? })
    }
    | "const" TypeArguments "[" ElementList CommaOpt "]" {
        Ok(Node::ListLiteral { element_list: $4? })
    }
    ;

SetOrMapLiteral -> Result<Node<'input>, ()>:
      "{" "}" {
        Ok(Node::SetOrMapLiteral { element_list: vec![] })
    }
    | "const" "{" "}" {
        Ok(Node::SetOrMapLiteral { element_list: vec![] })
    }
    | "{" ElementList CommaOpt "}" {
        Ok(Node::SetOrMapLiteral { element_list: $2? })
    }
    | "const" "{" ElementList CommaOpt "}" {
        Ok(Node::SetOrMapLiteral { element_list: $3? })
    }
    | TypeArguments "{" "}" {
        Ok(Node::SetOrMapLiteral { element_list: vec![] })
    }
    | "const" TypeArguments "{" "}" {
        Ok(Node::SetOrMapLiteral { element_list: vec![] })
    }
    | TypeArguments "{" ElementList CommaOpt "}" {
        Ok(Node::SetOrMapLiteral { element_list: $3? })
    }
    | "const" TypeArguments "{" ElementList CommaOpt "}" {
        Ok(Node::SetOrMapLiteral { element_list: $4? })
    }
    ;

ElementList -> Result<Vec<CollectionElement<'input>>, ()>:
      Element { Ok(vec![$1?]) }
    | ElementList "," Element { flatten($1, $3?) }
    ;

Element -> Result<CollectionElement<'input>, ()>:
      ExpressionElement { $1 }
    | MapElement { $1 }
    ;

ExpressionElement -> Result<CollectionElement<'input>, ()>:
    Expression {
        Ok(CollectionElement::ExpressionElement { expr: Box::new($1?) })
    }
    ;

MapElement -> Result<CollectionElement<'input>, ()>:
    Expression ":" Expression {
        Ok(CollectionElement::MapElement { key_expr: Box::new($1?), value_expr: Box::new($3?) })
    }
    ;

ThrowExpression -> Result<Node<'input>, ()>:
    "throw" Expression {
        Ok(Node::ThrowExpression { expr: Box::new($2?) })
    }
    ;

ThisExpression -> Result<Node<'input>, ()>:
    "this" { Ok(Node::ThisExpression) }
    ;

ConditionalExpression -> Result<Node<'input>, ()>:
      IfNullExpression { $1 }
    | IfNullExpression "?" Expression ":" Expression {
        Ok(Node::ConditionalExpression { condition: Box::new($1?), if_true_expr: Box::new($3?), if_false_expr: Box::new($5?) })
    }
    ;

ConditionalExpressionNotBrace -> Result<Node<'input>, ()>:
      IfNullExpressionNotBrace { $1 }
    | IfNullExpressionNotBrace "?" Expression ":" Expression {
        Ok(Node::ConditionalExpression { condition: Box::new($1?), if_true_expr: Box::new($3?), if_false_expr: Box::new($5?) })
    }
    ;

IfNullExpression -> Result<Node<'input>, ()>:
      LogicalOrExpression { $1 }
    | IfNullExpression "??" LogicalOrExpression{
        Ok(Node::BinaryExpression { operator: "??", left: Box::new($1?), right: Box::new($3?) })
    }
    ;

IfNullExpressionNotBrace -> Result<Node<'input>, ()>:
      LogicalOrExpressionNotBrace { $1 }
    | IfNullExpressionNotBrace "??" LogicalOrExpression{
        Ok(Node::BinaryExpression { operator: "??", left: Box::new($1?), right: Box::new($3?) })
    }
    ;

LogicalOrExpression -> Result<Node<'input>, ()>:
      LogicalAndExpression { $1 }
    | LogicalOrExpression "||" LogicalAndExpression{
        Ok(Node::BinaryExpression { operator: "||", left: Box::new($1?), right: Box::new($3?) })
    }
    ;

LogicalOrExpressionNotBrace -> Result<Node<'input>, ()>:
      LogicalAndExpressionNotBrace { $1 }
    | LogicalOrExpressionNotBrace "||" LogicalAndExpression{
        Ok(Node::BinaryExpression { operator: "||", left: Box::new($1?), right: Box::new($3?) })
    }
    ;

LogicalAndExpression -> Result<Node<'input>, ()>:
      EqualityExpression { $1 }
    | LogicalAndExpression "&&" EqualityExpression{
        Ok(Node::BinaryExpression { operator: "&&", left: Box::new($1?), right: Box::new($3?) })
    }
    ;

LogicalAndExpressionNotBrace -> Result<Node<'input>, ()>:
      EqualityExpressionNotBrace { $1 }
    | LogicalAndExpressionNotBrace "&&" EqualityExpression{
        Ok(Node::BinaryExpression { operator: "&&", left: Box::new($1?), right: Box::new($3?) })
    }
    ;

EqualityExpression -> Result<Node<'input>, ()>:
      RelationalExpression "==" RelationalExpression {
        Ok(Node::BinaryExpression { operator: "==", left: Box::new($1?), right: Box::new($3?) })
    }
    | RelationalExpression "!=" RelationalExpression {
        Ok(Node::BinaryExpression { operator: "!=", left: Box::new($1?), right: Box::new($3?) })
    }
    | RelationalExpression { $1 }
    ;

EqualityExpressionNotBrace -> Result<Node<'input>, ()>:
      RelationalExpressionNotBrace "==" RelationalExpression {
        Ok(Node::BinaryExpression { operator: "==", left: Box::new($1?), right: Box::new($3?) })
    }
    | RelationalExpressionNotBrace "!=" RelationalExpression {
        Ok(Node::BinaryExpression { operator: "!=", left: Box::new($1?), right: Box::new($3?) })
    }
    | RelationalExpressionNotBrace { $1 }
    ;

RelationalExpression -> Result<Node<'input>, ()>:
      BitwiseOrExpression ">=" BitwiseOrExpression {
        Ok(Node::BinaryExpression { operator: ">=", left: Box::new($1?), right: Box::new($3?) })
    }
    | BitwiseOrExpression ">" BitwiseOrExpression {
        Ok(Node::BinaryExpression { operator: ">", left: Box::new($1?), right: Box::new($3?) })
    }
    | BitwiseOrExpression "<=" BitwiseOrExpression {
        Ok(Node::BinaryExpression { operator: "<=", left: Box::new($1?), right: Box::new($3?) })
    }
    | BitwiseOrExpression "<" BitwiseOrExpression {
        Ok(Node::BinaryExpression { operator: "<", left: Box::new($1?), right: Box::new($3?) })
    }
    | BitwiseOrExpression TypeTest {
        Ok(Node::TypeTestExpression { child: Box::new($1?), type_test: $2? })
    }
    | BitwiseOrExpression TypeCast {
        Ok(Node::TypeCastExpression { child: Box::new($1?), type_cast: $2? })
    }
    | BitwiseOrExpression { $1 }
    ;

RelationalExpressionNotBrace -> Result<Node<'input>, ()>:
      BitwiseOrExpressionNotBrace ">=" BitwiseOrExpression {
        Ok(Node::BinaryExpression { operator: ">=", left: Box::new($1?), right: Box::new($3?) })
    }
    | BitwiseOrExpressionNotBrace ">" BitwiseOrExpression {
        Ok(Node::BinaryExpression { operator: ">", left: Box::new($1?), right: Box::new($3?) })
    }
    | BitwiseOrExpressionNotBrace "<=" BitwiseOrExpression {
        Ok(Node::BinaryExpression { operator: "<=", left: Box::new($1?), right: Box::new($3?) })
    }
    | BitwiseOrExpressionNotBrace "<" BitwiseOrExpression {
        Ok(Node::BinaryExpression { operator: "<", left: Box::new($1?), right: Box::new($3?) })
    }
    | BitwiseOrExpressionNotBrace TypeTest {
        Ok(Node::TypeTestExpression { child: Box::new($1?), type_test: $2? })
    }
    | BitwiseOrExpressionNotBrace TypeCast {
        Ok(Node::TypeCastExpression { child: Box::new($1?), type_cast: $2? })
    }
    | BitwiseOrExpressionNotBrace { $1 }
    ;

BitwiseOrExpression -> Result<Node<'input>, ()>:
      BitwiseOrExpression "|" BitwiseXorExpression {
        Ok(Node::BinaryExpression { operator: "|", left: Box::new($1?), right: Box::new($3?) })
    }
    | BitwiseXorExpression { $1 }
    ;

BitwiseOrExpressionNotBrace -> Result<Node<'input>, ()>:
      BitwiseOrExpressionNotBrace "|" BitwiseXorExpression {
        Ok(Node::BinaryExpression { operator: "|", left: Box::new($1?), right: Box::new($3?) })
    }
    | BitwiseXorExpressionNotBrace { $1 }
    ;

BitwiseXorExpression -> Result<Node<'input>, ()>:
      BitwiseXorExpression "^" BitwiseAndExpression {
        Ok(Node::BinaryExpression { operator: "^", left: Box::new($1?), right: Box::new($3?) })
    }
    | BitwiseAndExpression { $1 }
    ;

BitwiseXorExpressionNotBrace -> Result<Node<'input>, ()>:
      BitwiseXorExpressionNotBrace "^" BitwiseAndExpression {
        Ok(Node::BinaryExpression { operator: "^", left: Box::new($1?), right: Box::new($3?) })
    }
    | BitwiseAndExpressionNotBrace { $1 }
    ;

BitwiseAndExpression -> Result<Node<'input>, ()>:
      BitwiseAndExpression "&" ShiftExpression {
        Ok(Node::BinaryExpression { operator: "&", left: Box::new($1?), right: Box::new($3?) })
    }
    | ShiftExpression { $1 }
    ;

BitwiseAndExpressionNotBrace -> Result<Node<'input>, ()>:
      BitwiseAndExpressionNotBrace "&" ShiftExpression {
        Ok(Node::BinaryExpression { operator: "&", left: Box::new($1?), right: Box::new($3?) })
    }
    | ShiftExpressionNotBrace { $1 }
    ;

ShiftExpression -> Result<Node<'input>, ()>:
      ShiftExpression "<<" AdditiveExpression {
        Ok(Node::BinaryExpression { operator: "<<", left: Box::new($1?), right: Box::new($3?) })
    }
    | ShiftExpression ">>" AdditiveExpression {
        Ok(Node::BinaryExpression { operator: ">>", left: Box::new($1?), right: Box::new($3?) })
    }
    | AdditiveExpression { $1 }
    ;

ShiftExpressionNotBrace -> Result<Node<'input>, ()>:
      ShiftExpressionNotBrace "<<" AdditiveExpression {
        Ok(Node::BinaryExpression { operator: "<<", left: Box::new($1?), right: Box::new($3?) })
    }
    | ShiftExpressionNotBrace ">>" AdditiveExpression {
        Ok(Node::BinaryExpression { operator: ">>", left: Box::new($1?), right: Box::new($3?) })
    }
    | AdditiveExpressionNotBrace { $1 }
    ;

AdditiveExpression -> Result<Node<'input>, ()>:
      AdditiveExpression '+' MultiplicativeExpression { 
        Ok(Node::BinaryExpression { operator: "+", left: Box::new($1?), right: Box::new($3?) })
    }
    | AdditiveExpression '-' MultiplicativeExpression { 
        Ok(Node::BinaryExpression { operator: "-", left: Box::new($1?), right: Box::new($3?) })
    }
    | MultiplicativeExpression { $1 }
    ;

AdditiveExpressionNotBrace -> Result<Node<'input>, ()>:
      AdditiveExpressionNotBrace '+' MultiplicativeExpression { 
        Ok(Node::BinaryExpression { operator: "+", left: Box::new($1?), right: Box::new($3?) })
    }
    | AdditiveExpressionNotBrace '-' MultiplicativeExpression { 
        Ok(Node::BinaryExpression { operator: "-", left: Box::new($1?), right: Box::new($3?) })
    }
    | MultiplicativeExpressionNotBrace { $1 }
    ;

MultiplicativeExpression -> Result<Node<'input>, ()>:
      MultiplicativeExpression '*' Primary { 
        Ok(Node::BinaryExpression { operator: "*", left: Box::new($1?), right: Box::new($3?) })
    }
    | MultiplicativeExpression '/' Primary { 
        Ok(Node::BinaryExpression { operator: "/", left: Box::new($1?), right: Box::new($3?) })
    }
    | MultiplicativeExpression '%' Primary { 
        Ok(Node::BinaryExpression { operator: "%", left: Box::new($1?), right: Box::new($3?) })
    }
    | MultiplicativeExpression '~/' Primary { 
        Ok(Node::BinaryExpression { operator: "~/", left: Box::new($1?), right: Box::new($3?) })
    }
    | UnaryExpression { $1 }
    ;

MultiplicativeExpressionNotBrace -> Result<Node<'input>, ()>:
      MultiplicativeExpressionNotBrace '*' Primary { 
        Ok(Node::BinaryExpression { operator: "*", left: Box::new($1?), right: Box::new($3?) })
    }
    | MultiplicativeExpressionNotBrace '/' Primary { 
        Ok(Node::BinaryExpression { operator: "/", left: Box::new($1?), right: Box::new($3?) })
    }
    | MultiplicativeExpressionNotBrace '%' Primary { 
        Ok(Node::BinaryExpression { operator: "%", left: Box::new($1?), right: Box::new($3?) })
    }
    | MultiplicativeExpressionNotBrace '~/' Primary { 
        Ok(Node::BinaryExpression { operator: "~/", left: Box::new($1?), right: Box::new($3?) })
    }
    | UnaryExpressionNotBrace { $1 }
    ;

UnaryExpression -> Result<Node<'input>, ()>:
      "-" UnaryExpression {
        Ok(Node::UnaryOpExpression { operator: "-", child: Box::new($2?) })
    }
    | "!" UnaryExpression {
        Ok(Node::UnaryOpExpression { operator: "!", child: Box::new($2?) })
    }
    | "~" UnaryExpression {
        Ok(Node::UnaryOpExpression { operator: "~", child: Box::new($2?) })
    }
    | "++" SelectorExpression {
        Ok(Node::UpdateExpression { operator: "++", is_prefix: true, child: Box::new($2?) })
    }
    | "--" SelectorExpression {
        Ok(Node::UpdateExpression { operator: "--", is_prefix: true, child: Box::new($2?) })
    }
    | PostfixExpression { $1 }
    ;

UnaryExpressionNotBrace -> Result<Node<'input>, ()>:
      "-" UnaryExpression {
        Ok(Node::UnaryOpExpression { operator: "-", child: Box::new($2?) })
    }
    | "!" UnaryExpression {
        Ok(Node::UnaryOpExpression { operator: "!", child: Box::new($2?) })
    }
    | "~" UnaryExpression {
        Ok(Node::UnaryOpExpression { operator: "~", child: Box::new($2?) })
    }
    | "++" SelectorExpression {
        Ok(Node::UpdateExpression { operator: "++", is_prefix: true, child: Box::new($2?) })
    }
    | "--" SelectorExpression {
        Ok(Node::UpdateExpression { operator: "--", is_prefix: true, child: Box::new($2?) })
    }
    | PostfixExpressionNotBrace { $1 }
    ;

PostfixExpression -> Result<Node<'input>, ()>:
      SelectorExpression { $1 }
    | Primary "++" {
        Ok(Node::UpdateExpression { operator: "++", is_prefix: false, child: Box::new($1?) })
    }
    | Primary "--" {
        Ok(Node::UpdateExpression { operator: "--", is_prefix: false, child: Box::new($1?) })
    }
    ;

PostfixExpressionNotBrace -> Result<Node<'input>, ()>:
      SelectorExpressionNotBrace { $1 }
    | PrimaryNotBrace "++" {
        Ok(Node::UpdateExpression { operator: "++", is_prefix: false, child: Box::new($1?) })
    }
    | PrimaryNotBrace "--" {
        Ok(Node::UpdateExpression { operator: "--", is_prefix: false, child: Box::new($1?) })
    }
    ;

SelectorExpression -> Result<Node<'input>, ()>:
      Primary { $1 }
    | SelectorExpression Selector {
        Ok(Node::SelectorExpression { child: Box::new($1?), selector: $2? })
    }
    ;

SelectorExpressionNotBrace -> Result<Node<'input>, ()>:
      PrimaryNotBrace { $1 }
    | SelectorExpressionNotBrace Selector {
        Ok(Node::SelectorExpression { child: Box::new($1?), selector: $2? })
    }
    ;

Selector -> Result<Selector<'input>, ()>:
      Arguments { 
        Ok(Selector::Args { args: Box::new($1?) })
    }
    | "." Identifier { 
        Ok(Selector::Attr { identifier: $2? }) 
    }
    | "." Identifier Arguments {
        Ok(Selector::Method { identifier: $2?, arguments: Box::new($3?) })
    }
    | "[" Expression "]" {
        Ok(Selector::Index { expr: Box::new($2?) })
    }
    ;

//ArgumentPart -> Result<Node<'input>, ()>:
//      Arguments { $1 }
//    | TypeArguments Arguments { $2 }
//    ;

Arguments -> Result<Node<'input>, ()>:
      "(" ")" { Ok(Node::Arguments { children: vec![] }) }
    | "(" ArgumentList CommaOpt ")" { Ok(Node::Arguments { children: $2? }) }
    ;

ArgumentList -> Result<Vec<CallParameter<'input>>, ()>:
      NamedArgument { Ok(vec![$1?]) }
    | NormalArgument { Ok(vec![$1?]) }
    | ArgumentList "," NamedArgument { flatten($1, $3?) }
    | ArgumentList "," NormalArgument { flatten($1, $3?) }
    ;

NamedArgument -> Result<CallParameter<'input>, ()>:
    Label Expression { Ok(CallParameter { identifier: Some($1?), expr: Box::new($2?) }) }
    ;

NormalArgument -> Result<CallParameter<'input>, ()>:
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

TypeTest -> Result<TypeTest<'input>, ()>:
      "is" TypeNotVoid { Ok(TypeTest { dart_type: $2?, check_matching: true }) }
    | "is" "!" TypeNotVoid { Ok(TypeTest { dart_type: $3?, check_matching: false }) }
    ;

TypeCast -> Result<DartType<'input>, ()>:
    "as" TypeNotVoid { $2 }
    ;



//----------------------------------------------------------------------
//----------------------------Statements--------------------------------
//----------------------------------------------------------------------
Statements -> Result<Vec<Box<Node<'input>>>, ()>:
      %empty { Ok(vec![]) }
    | Statements Statement { flatten($1, Box::new($2?)) }
    ;

Statement -> Result<Node<'input>, ()>:
      NonLabeledStatement { $1 }
    | Label NonLabeledStatement {
        Ok(Node::LabeledStatement { label: $1?, stmt: Box::new($2?) })
    }
    ;

NonLabeledStatement -> Result<Node<'input>, ()>:
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
    | ";" { Ok(Node::EmptyStatement) }
    ;

ExpressionStatement -> Result<Node<'input>, ()>:
    ExpressionNotBrace ";" { Ok(Node::ExpressionStatement { expr: Box::new($1?) }) }
    ;

LocalVariableDeclaration -> Result<Node<'input>, ()>:
    InitializedVariableDeclaration ";" {
        Ok(Node::VariableDeclarationList { decl_list: $1? })
    }
    ;

IfStatement -> Result<Node<'input>, ()>:
      "if" "(" Expression ")" Statement { Ok(Node::IfStatement { condition: Box::new($3?), if_true_stmt: Box::new($5?), if_false_stmt: None }) }
    | "if" "(" Expression ")" Statement "else" Statement { Ok(Node::IfStatement { condition: Box::new($3?), if_true_stmt: Box::new($5?), if_false_stmt: Some(Box::new($7?)) }) }
    ;

ForStatement -> Result<Node<'input>, ()>:
    "for" "(" ForLoopParts ")" Statement {
        let part = $3?;
        Ok(Node::ForStatement { init: part.0, condition: part.1, update: part.2, stmt: Box::new($5?) })
    }
    ;

ForLoopParts -> Result<(Option<Box<Node<'input>>>,Option<Box<Node<'input>>>,Option<Vec<Box<Node<'input>>>>), ()>:
      ForInitializerStatement ExpressionOpt ";" ExpressionListOpt {
        Ok(($1?, $2?, $4?))
      }
    ;

ForInitializerStatement -> Result<Option<Box<Node<'input>>>, ()>:
      LocalVariableDeclaration { Ok(Some(Box::new($1?))) }
    | ExpressionOpt ";" {
        match $1? {
            Some(v) => Ok(Some(Box::new(Node::ExpressionStatement { expr: v }))),
            None => Ok(None),
        }
     }
    ;

WhileStatement -> Result<Node<'input>, ()>:
    "while" "(" Expression ")" Statement {
        Ok(Node::WhileStatement { condition: Box::new($3?), stmt: Box::new($5?) })
    }
    ;

DoStatement -> Result<Node<'input>, ()>:
    "do" Statement "while" "(" Expression ")" ";" {
        Ok(Node::DoStatement { condition: Box::new($5?), stmt: Box::new($2?) })
    }
    ;

SwitchStatement -> Result<Node<'input>, ()>:
    "switch" "(" Expression ")" "{" SwitchCaseList DefaultCaseOpt "}" {
        Ok(Node::SwitchStatement { expr: Box::new($3?), case_list: $6?, default_case: $7? })
    }
    ;

SwitchCaseList -> Result<Vec<SwitchCase<'input>>, ()>:
      %empty { Ok(vec![]) }
    | SwitchCaseList SwitchCase { flatten($1, $2?) }
    ;

SwitchCase -> Result<SwitchCase<'input>, ()>:
      "case" Expression ":" Statements {
        Ok(SwitchCase { label_list: vec![], expr: Box::new($2?), stmt_list: $4? })
    }
    ;

DefaultCase -> Result<DefaultCase<'input>, ()>:
      "default" ":" Statements {
        Ok(DefaultCase { label_list: vec![], stmt_list: $3? })
    }
    ;

DefaultCaseOpt -> Result<Option<DefaultCase<'input>>, ()>:
      %empty { Ok(None) }
    | DefaultCase { Ok(Some($1?)) }
    ;

RethrowStatement -> Result<Node<'input>, ()>:
    "rethrow" ";" {
        Ok(Node::RethrowStatement)
    }
    ;

TryStatement -> Result<Node<'input>, ()>:
      "try" BlockStatement FinallyPart {
        Ok(Node::TryFinallyStatement { block_try: Box::new($2?), block_finally: Box::new($3?) })
    }
    | "try" BlockStatement OnPartList {
        Ok(Node::TryOnStatement { block_try: Box::new($2?), on_part_list: $3? })
    }
    | "try" BlockStatement OnPartList FinallyPart {
        Ok(Node::TryFinallyStatement { 
            
            block_try: Box::new(Node::TryOnStatement {
                block_try: Box::new($2?),
                on_part_list: $3?,
            }),
            block_finally: Box::new($4?), 
        })
    }
    ;

OnPartList -> Result<Vec<TryOnPart<'input>>, ()>:
      OnPart { Ok(vec![$1?]) }
    | OnPartList OnPart { flatten($1, $2?) }
    ;

OnPart -> Result<TryOnPart<'input>, ()>:
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

CatchPart -> Result<TryCatchPart<'input>, ()>:
      "catch" "(" Identifier ")" {
        Ok(TryCatchPart { id_error: $3?, id_trace: None })
    }
    | "catch" "(" Identifier "," Identifier ")" {
        Ok(TryCatchPart { id_error: $3?, id_trace: Some($5?) })
    }
    ;

FinallyPart -> Result<Node<'input>, ()>:
    "finally" BlockStatement { $2 }
    ;

ReturnStatement -> Result<Node<'input>, ()>:
      "return" ";" { Ok(Node::ReturnStatement { value: None }) }
    | "return" Expression ";" {
        Ok(Node::ReturnStatement { value: Some(Box::new($2?)) })
    }
    ;

Label -> Result<Identifier<'input>, ()>:
    Identifier ":" { $1 }
    ;

BreakStatement -> Result<Node<'input>, ()>:
      "break" ";" {
        Ok(Node::BreakStatement { label: None })
    }
    | "break" Identifier ";" {
        Ok(Node::BreakStatement { label: Some($2?) })
    }
    ;

ContinueStatement -> Result<Node<'input>, ()>:
      "continue" ";" {
        Ok(Node::ContinueStatement { label: None })
    }
    | "continue" Identifier ";" {
        Ok(Node::ContinueStatement { label: Some($2?) })
    }
    ;




//----------------------------------------------------------------------
//-----------------------Libraries and Scripts--------------------------
//----------------------------------------------------------------------
TopLevelDeclaration -> Result<Node<'input>, ()>:
      ClassDeclaration { $1 }
    | TopFunctionDeclaration { $1 }
    | TopVariableDeclaration { $1 }
    ;

LibraryDeclaration -> Result<LibraryDeclaration<'input>, ()>:
    LibraryImportList TopLevelDeclarationList { 
        Ok(LibraryDeclaration { import_list: $1?, top_level_declaration_list: $2? })
    }
    ;

TopLevelDeclarationList -> Result<Vec<Box<Node<'input>>>, ()>:
      %empty { Ok(vec![]) }
    | TopLevelDeclarationList TopLevelDeclaration { flatten($1, Box::new($2?)) }
    ;

LibraryImportList -> Result<Vec<LibraryImport<'input>>, ()>:
      %empty { Ok(vec![]) }
    | LibraryImportList LibraryImport { flatten($1, $2?) }
    ;

LibraryImport -> Result<LibraryImport<'input>, ()>:
      "import" Uri ";" { Ok(LibraryImport { uri: $2?, identifier: None }) }
    | "import" Uri "as" Identifier ";" { Ok(LibraryImport { uri: $2?, identifier: Some($4?) }) }
    ;

Uri -> Result<&'input str, ()>:
    "STRING" { Ok($lexer.span_str($span)) }
    ;

TopFunctionDeclaration -> Result<Node<'input>, ()>:
    FunctionSignature FunctionBody {
        Ok(Node::FunctionDeclaration { signature: $1?, body: Box::new($2?) })
    }
    ;

TopVariableDeclaration -> Result<Node<'input>, ()>:
      "var" InitializedIdentifierList ";" { 
        Ok(Node::VariableDeclarationList { decl_list: $2? }) 
    }
    | Type InitializedIdentifierList ";" { 
        Ok(Node::VariableDeclarationList { decl_list: $2? }) 
    }
    | "late" "var" InitializedIdentifierList ";" { 
        Ok(Node::VariableDeclarationList { decl_list: $3? }) 
    }
    | "late" Type InitializedIdentifierList ";" { 
        Ok(Node::VariableDeclarationList { decl_list: $3? }) 
    }
    | "late" "final" InitializedIdentifierList ";" { 
        Ok(Node::VariableDeclarationList { decl_list: $3? }) 
    }
    | "late" "final" Type InitializedIdentifierList ";" { 
        Ok(Node::VariableDeclarationList { decl_list: $4? }) 
    }
    ;




//----------------------------------------------------------------------
//--------------------------Static Types--------------------------------
//----------------------------------------------------------------------
Type -> Result<DartType<'input>, ()>:
    TypeNotFunction { $1 }
    ;

TypeNotVoid -> Result<DartType<'input>, ()>:
    TypeNotVoidNotFunction { $1 }
    ;

TypeNotFunction -> Result<DartType<'input>, ()>:
      "void" { Ok(DartType::Void) }
    | TypeNotVoidNotFunction { $1 }
    ;

TypeNotVoidNotFunction -> Result<DartType<'input>, ()>:
      TypeName { 
        Ok(DartType::Named { type_name: $1?, type_arguments: vec![], is_nullable: false }) }
    ;

TypeName -> Result<DartTypeName<'input>, ()>:
      Identifier {
        Ok(DartTypeName { identifier: $1?, module: None })
    }
    ;

TypeArguments -> Result<Vec<DartType<'input>>, ()>:
    "<" TypeList ">" { $2 }
    ;
TypeList -> Result<Vec<DartType<'input>>, ()>:
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

#[derive(Debug)]
pub enum Node<'input> {
    BinaryExpression {
        operator: &'static str,
        left: Box<Node<'input>>,
        right: Box<Node<'input>>,
    },
    ConditionalExpression {
        condition: Box<Node<'input>>,
        if_true_expr: Box<Node<'input>>,
        if_false_expr: Box<Node<'input>>,
    },
    UnaryOpExpression {
        operator: &'static str,
        child: Box<Node<'input>>,
    },
    UpdateExpression {
        operator: &'static str,
        is_prefix: bool,
        child: Box<Node<'input>>,
    },
    AssignmentExpression {
        operator: &'static str,
        left: Box<Node<'input>>,
        right: Box<Node<'input>>,
    },
    TypeTestExpression {
        child: Box<Node<'input>>,
        type_test: TypeTest<'input>,
    },
    TypeCastExpression {
        child: Box<Node<'input>>,
        type_cast: DartType<'input>,
    },
    NumericLiteral {
        value: &'input str,
    },
    StringLiteral {
        literal_list: Vec<&'input str>,
    },
    BooleanLiteral {
        value: &'input str,
    },
    NullLiteral,
    ListLiteral {
        element_list: Vec<CollectionElement<'input>>,
    },
    SetOrMapLiteral {
        element_list: Vec<CollectionElement<'input>>,
    },
    IdentifierNode {
        identifier: Identifier<'input>,
    },
    Arguments {
        children: Vec<CallParameter<'input>>
    },
    SelectorExpression {
        child: Box<Node<'input>>,
        selector: Selector<'input>,
    },
    ThrowExpression {
        expr: Box<Node<'input>>,
    },
    ThisExpression,

    LabeledStatement {
        label: Identifier<'input>,
        stmt: Box<Node<'input>>,
    },
    BlockStatement {
        children: Vec<Box<Node<'input>>>,
    },
    ExpressionStatement {
        expr: Box<Node<'input>>,
    },
    EmptyStatement,
    VariableDeclarationList {
        decl_list: Vec<VariableDeclaration<'input>>,
    },
    IfStatement {
        condition: Box<Node<'input>>,
        if_true_stmt: Box<Node<'input>>,
        if_false_stmt: Option<Box<Node<'input>>>,
    },
    RethrowStatement,
    TryFinallyStatement {
        block_try: Box<Node<'input>>,
        block_finally: Box<Node<'input>>,
    },
    TryOnStatement {
        block_try: Box<Node<'input>>,
        on_part_list: Vec<TryOnPart<'input>>,
    },
    ForStatement {
        init: Option<Box<Node<'input>>>,
        condition: Option<Box<Node<'input>>>,
        update: Option<Vec<Box<Node<'input>>>>,
        stmt: Box<Node<'input>>,
    },
    WhileStatement {
        condition: Box<Node<'input>>,
        stmt: Box<Node<'input>>,
    },
    DoStatement {
        condition: Box<Node<'input>>,
        stmt: Box<Node<'input>>,
    },
    BreakStatement {
        label: Option<Identifier<'input>>,
    },
    ContinueStatement {
        label: Option<Identifier<'input>>,
    },
    ReturnStatement {
        value: Option<Box<Node<'input>>>,
    },
    SwitchStatement {
        expr: Box<Node<'input>>,
        case_list: Vec<SwitchCase<'input>>,
        default_case: Option<DefaultCase<'input>>,
    },
    FunctionDeclaration {
        signature: FunctionSignature<'input>,
        body: Box<Node<'input>>,
    },
    ClassDeclaration {
        identifier: Identifier<'input>,
        member_list: Vec<Member<'input>>,
    },
}

#[derive(Debug)]
pub struct LibraryDeclaration<'input> {
    pub import_list: Vec<LibraryImport<'input>>,
    pub top_level_declaration_list: Vec<Box<Node<'input>>>,
}

#[derive(Debug)]
pub struct LibraryImport<'input> {
    pub uri: &'input str,
    pub identifier: Option<Identifier<'input>>,
}

#[derive(Debug)]
pub struct FunctionParameter<'input> {
    pub identifier: Identifier<'input>,
    pub expr: Option<Box<Node<'input>>>,
}

#[derive(Debug)]
pub struct SwitchCase<'input> {
    pub label_list: Vec<Box<Node<'input>>>,
    pub expr: Box<Node<'input>>,
    pub stmt_list: Vec<Box<Node<'input>>>,
}

#[derive(Debug)]
pub struct DefaultCase<'input> {
    pub label_list: Vec<Box<Node<'input>>>,
    pub stmt_list: Vec<Box<Node<'input>>>,
}

#[derive(Debug)]
pub struct TryOnPart<'input> {
    pub catch_part: Option<TryCatchPart<'input>>,
    pub exc_type: Option<DartType<'input>>,
    pub block: Box<Node<'input>>,
}

#[derive(Debug)]
pub struct TryCatchPart<'input> {
    pub id_error: Identifier<'input>,
    pub id_trace: Option<Identifier<'input>>,
}

#[derive(Debug)]
pub enum CollectionElement<'input> {
    ExpressionElement {
        expr: Box<Node<'input>>,
    },
    MapElement {
        key_expr: Box<Node<'input>>,
        value_expr: Box<Node<'input>>,
    },
}

#[derive(Debug)]
pub enum Selector<'input> {
    Index {
        expr: Box<Node<'input>>,
    },
    Attr {
        identifier: Identifier<'input>,
    },
    Method {
        identifier: Identifier<'input>,
        arguments: Box<Node<'input>>,
    },
    Args {
        args: Box<Node<'input>>,
    }
}

#[derive(Debug)]
pub struct TypeTest<'input> {
    pub dart_type: DartType<'input>,
    pub check_matching: bool,
}

#[derive(Debug)]
pub enum DartType<'input> {
    Named {
        type_name: DartTypeName<'input>,
        type_arguments: Vec<DartType<'input>>,
        is_nullable: bool,
    },
    Void,
}

#[derive(Debug)]
pub struct DartTypeName<'input> {
    pub identifier: Identifier<'input>,
    pub module: Option<Identifier<'input>>,
}

#[derive(Debug)]
pub struct FunctionSignature<'input> {
    pub return_type: Option<DartType<'input>>,
    pub name: Identifier<'input>,
    pub param: FunctionParamSignature<'input>,
}

#[derive(Debug)]
pub struct VariableDeclaration<'input> {
    pub identifier: Identifier<'input>,
    pub expr: Option<Box<Node<'input>>>,
}

#[derive(Debug)]
pub struct FunctionParamSignature<'input> {
    pub normal_list: Vec<FunctionParameter<'input>>,
    pub option_list: Vec<FunctionParameter<'input>>,
    pub named_list: Vec<FunctionParameter<'input>>,
}

#[derive(Debug)]
pub struct CallParameter<'input> {
    pub identifier: Option<Identifier<'input>>,
    pub expr: Box<Node<'input>>,
}

#[derive(Debug)]
pub struct Identifier<'input> {
    pub value: &'input str,
}

#[derive(Debug)]
pub struct ConstructorSignature<'input> {
    pub name: Option<Identifier<'input>>,
    pub param: FunctionParamSignature<'input>,
}

#[derive(Debug)]
pub enum Member<'input> {
    MethodImpl {
        signature: FunctionSignature<'input>,
        body: Box<Node<'input>>,
    },
    ConstructorImpl {
        signature: ConstructorSignature<'input>,
        body: Box<Node<'input>>,
    },
    VariableDecl {
        decl_list: Vec<VariableDeclaration<'input>>,
    }
}
