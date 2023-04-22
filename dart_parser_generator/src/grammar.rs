use crate::parser_generator_lr0::ProductionRuleData;

pub const START_SYMBOL: &'static str = "LibraryDeclaration";

pub const EPSILON: &'static str = "[EMPTY]";
pub const END: &'static str = "[END]";

const DART_GRAMMARS: [&'static str; 136] = [
// Variables
"InitializedVariableDeclaration ::= DeclaredIdentifier
    |/ DeclaredIdentifier '=' Expression
    |/ InitializedVariableDeclaration ',' InitializedIdentifier",
"InitializedIdentifier ::= Identifier
    |/ Identifier '=' Expression",
"InitializedIdentifierList ::= InitializedIdentifier
    |/ InitializedIdentifierList ',' InitializedIdentifier",
// Functions
"FunctionSignature ::= Identifier FormalParameterList
    |/ Type Identifier FormalParameterList",
"FunctionBody ::= BlockStatement
    |/ '=>' Expression ';'",
"BlockStatement ::= '{' Statements '}'",
"FormalParameterList ::= '(' ')'
    |/ '(' NormalFormalParameterList CommaOpt ')'
    |/ '(' NormalFormalParameterList ',' OptionalOrNamedFormalParameterList ')'
    |/ '(' OptionalOrNamedFormalParameterList ')'",
"NormalFormalParameterList ::= NormalFormalParameter
    |/ NormalFormalParameterList ',' NormalFormalParameter",
"OptionalOrNamedFormalParameterList ::= OptionalPositionalFormalParameterList
    |/ NamedFormalParameterList",
"OptionalPositionalFormalParameterList ::= '[' OptionalPositionalFormalParameterListInternal CommaOpt ']'",
"OptionalPositionalFormalParameterListInternal ::= DefaultFormalParameter
    |/ OptionalPositionalFormalParameterListInternal ',' DefaultFormalParameter",
"NamedFormalParameterList ::= '{' NamedFormalParameterListInternal CommaOpt '}'",
"NamedFormalParameterListInternal ::= DefaultNamedParameter
    |/ NamedFormalParameterListInternal ',' DefaultNamedParameter",
"NormalFormalParameter ::= Identifier
    |/ DeclaredIdentifier",
"DefaultFormalParameter ::= NormalFormalParameter
    |/ NormalFormalParameter '=' Expression",
"DefaultNamedParameter ::= Identifier
    |/ DeclaredIdentifier
    |/ DeclaredIdentifier '=' Expression
    |/ Identifier ':' Expression
    |/ 'required' Identifier
    |/ 'required' DeclaredIdentifier
    |/ 'required' DeclaredIdentifier '=' Expression
    |/ 'required' Identifier ':' Expression",
"DeclaredIdentifier ::= 'var' Identifier
    |/ Type Identifier
    |/ 'late' 'var' Identifier
    |/ 'late' Type Identifier
    |/ 'const' Identifier
    |/ 'const' Type Identifier
    |/ 'final' Identifier
    |/ 'final' Type Identifier
    |/ 'late' 'final' Identifier
    |/ 'late' 'final' Type Identifier",
// Classes
"ClassDeclaration ::= 'class' Identifier '{' '}'
    |/ 'class' Identifier '{' ClassDeclarationInternal '}'",
"ClassDeclarationInternal ::= ClassMemberDeclaration
    |/ ClassDeclarationInternal ClassMemberDeclaration",
"ClassMemberDeclaration ::= Declaration ';'
    |/ MemberImpl",
"MemberImpl ::= FunctionSignature FunctionBody",
"Declaration ::= 'var' InitializedIdentifierList
    |/ Type InitializedIdentifierList
    |/ 'late' 'var' InitializedIdentifierList
    |/ 'late' Type InitializedIdentifierList
    |/ 'final' InitializedIdentifierList
    |/ 'final' Type InitializedIdentifierList
    |/ 'late' 'final' InitializedIdentifierList
    |/ 'late' 'final' Type InitializedIdentifierList",
// Expressions
"Expression ::= SelectorExpression AssignmentOperator Expression
    |/ ConditionalExpression
    |/ ThrowExpression",
"ExpressionNotBrace ::= SelectorExpressionNotBrace AssignmentOperator Expression
    |/ ConditionalExpressionNotBrace
    |/ ThrowExpression",
"AssignmentOperator ::= '=' |/ '*=' |/ '/=' |/ '~/=' |/ '%=' |/ '+=' |/ '-=' |/ '<<=' |/ '>>=' |/ '&=' |/ '^=' |/ '|=' |/ '??='",
"ExpressionOpt ::= [EMPTY]
    |/ Expression",
"ExpressionList ::= Expression
    |/ ExpressionList ',' Expression",
"ExpressionListOpt ::= [EMPTY]
    |/ ExpressionList",
"PrimaryExpression ::= '(' Expression ')'
    |/ 'NULL'
    |/ 'BOOLEAN'
    |/ 'NUMBER'
    |/ ThisExpression
    |/ StringLiteralList
    |/ ListLiteral
    |/ SetOrMapLiteral
    |/ Identifier",
"PrimaryExpressionNotBrace ::= '(' Expression ')'
    |/ 'NULL'
    |/ 'BOOLEAN'
    |/ 'NUMBER'
    |/ ThisExpression
    |/ StringLiteralList
    |/ ListLiteral
    |/ SetOrMapLiteralNotBrace
    |/ Identifier",
"StringLiteralList ::= StringLiteral
    |/ StringLiteralList StringLiteral",
"ListLiteral ::= '[' ']'
    |/ 'const' '[' ']'
    |/ '[' ElementList CommaOpt ']'
    |/ 'const' '[' ElementList CommaOpt ']'
    |/ TypeArguments '[' ']'
    |/ 'const' TypeArguments '[' ']'
    |/ TypeArguments '[' ElementList CommaOpt ']'
    |/ 'const' TypeArguments '[' ElementList CommaOpt ']'",
"SetOrMapLiteral ::= 'const' '{' '}'",
"SetOrMapLiteral ::= '{' '}'
    |/ 'const' '{' '}'
    |/ '{' ElementList CommaOpt '}'
    |/ 'const' '{' ElementList CommaOpt '}'
    |/ TypeArguments '{' '}'
    |/ 'const' TypeArguments '{' '}'
    |/ TypeArguments '{' ElementList CommaOpt '}'
    |/ 'const' TypeArguments '{' ElementList CommaOpt '}'",
"SetOrMapLiteralNotBrace ::= 'const' '{' '}'
    |/ 'const' '{' ElementList CommaOpt '}'
    |/ TypeArguments '{' '}'
    |/ 'const' TypeArguments '{' '}'
    |/ TypeArguments '{' ElementList CommaOpt '}'
    |/ 'const' TypeArguments '{' ElementList CommaOpt '}'",
"ElementList ::= Element
    |/ ElementList ',' Element",
"Element ::= ExpressionElement
    |/ MapElement",
"ExpressionElement ::= Expression",
"MapElement ::= Expression ':' Expression",
"ThrowExpression ::= 'throw' Expression",
"ThisExpression ::= 'this'",
"ConditionalExpression ::= IfNullExpression
    |/ IfNullExpression '?' Expression : Expression",
"ConditionalExpressionNotBrace ::= IfNullExpressionNotBrace
    |/ IfNullExpressionNotBrace '?' Expression : Expression",
"IfNullExpression ::= LogicalOrExpression
    |/ IfNullExpression '??' LogicalOrExpression",
"IfNullExpressionNotBrace ::= LogicalOrExpressionNotBrace
    |/ IfNullExpressionNotBrace '??' LogicalOrExpression", 
"LogicalOrExpression ::= LogicalAndExpression
    |/ LogicalOrExpression '||' LogicalAndExpression",
"LogicalOrExpressionNotBrace ::= LogicalAndExpressionNotBrace
    |/ LogicalOrExpressionNotBrace '||' LogicalAndExpression",
"LogicalAndExpression ::= EqualityExpression
    |/ LogicalAndExpression '&&' EqualityExpression",
"LogicalAndExpressionNotBrace ::= EqualityExpressionNotBrace
    |/ LogicalAndExpressionNotBrace '&&' EqualityExpression",
"EqualityExpression ::= RelationalExpression
    |/ RelationalExpression EqualityOperator RelationalExpression",
"EqualityExpressionNotBrace ::= RelationalExpressionNotBrace
    |/ RelationalExpressionNotBrace EqualityOperator RelationalExpression",
"EqualityOperator ::= '==' |/ '!='",
"RelationalExpression ::= BitwiseOrExpression
    |/ BitwiseOrExpression RelationalOperator BitwiseOrExpression
    |/ BitwiseOrExpression TypeTest
    |/ BitwiseOrExpression TypeCast",
"RelationalExpressionNotBrace ::= BitwiseOrExpressionNotBrace
    |/ BitwiseOrExpressionNotBrace RelationalOperator BitwiseOrExpression
    |/ BitwiseOrExpressionNotBrace TypeTest
    |/ BitwiseOrExpressionNotBrace TypeCast",
"RelationalOperator ::= '<' |/ '>' |/ '<=' |/ '>='",
"BitwiseOrExpression ::= BitwiseXorExpression
    |/ BitwiseOrExpression '|' BitwiseXorExpression",
"BitwiseOrExpressionNotBrace ::= BitwiseXorExpressionNotBrace
    |/ BitwiseOrExpressionNotBrace '|' BitwiseXorExpression",
"BitwiseXorExpression ::= BitwiseAndExpression
    |/ BitwiseXorExpression '^' BitwiseAndExpression",
"BitwiseXorExpressionNotBrace ::= BitwiseAndExpressionNotBrace
    |/ BitwiseXorExpressionNotBrace '^' BitwiseAndExpression",
"BitwiseAndExpression ::= ShiftExpression
    |/ BitwiseAndExpression '&' ShiftExpression",
"BitwiseAndExpressionNotBrace ::= ShiftExpressionNotBrace
    |/ BitwiseAndExpressionNotBrace '&' ShiftExpression",
"ShiftExpression ::= AdditiveExpression
    |/ ShiftExpression ShiftOperator AdditiveExpression",
"ShiftExpressionNotBrace ::= AdditiveExpressionNotBrace
    |/ ShiftExpressionNotBrace ShiftOperator AdditiveExpression",
"ShiftOperator ::= '<<' |/ '>>'",
"AdditiveExpression ::= AdditiveExpression '+' MultiplicativeExpression
    |/ AdditiveExpression '-' MultiplicativeExpression
    |/ MultiplicativeExpression",
"AdditiveExpressionNotBrace ::= AdditiveExpressionNotBrace '+' MultiplicativeExpression
    |/ AdditiveExpressionNotBrace '-' MultiplicativeExpression
    |/ MultiplicativeExpressionNotBrace",
"MultiplicativeExpression ::= MultiplicativeExpression MultiplicativeOperator PrimaryExpression
    |/ UnaryExpression",
"MultiplicativeExpressionNotBrace ::= MultiplicativeExpressionNotBrace MultiplicativeOperator PrimaryExpression
    |/ UnaryExpressionNotBrace",
"MultiplicativeOperator ::= '*' |/ '/' |/ '%' |/ '~/'",
"UnaryExpression ::= PostfixExpression
    |/ PrefixOperator UnaryExpression
    |/ IncrementOperator SelectorExpression",
"UnaryExpressionNotBrace ::= PostfixExpressionNotBrace
    |/ PrefixOperator UnaryExpression
    |/ IncrementOperator SelectorExpression",
"PrefixOperator ::= '!' |/ '-' |/ '~'",
"IncrementOperator ::= '++' |/ '--'",
"PostfixExpression ::= SelectorExpression
    |/ PrimaryExpression IncrementOperator",
"PostfixExpressionNotBrace ::= SelectorExpressionNotBrace
    |/ PrimaryExpressionNotBrace IncrementOperator",
"SelectorExpression ::= PrimaryExpression
    |/ SliceExpression
    |/ SelectorExpression Selector",
"SelectorExpressionNotBrace ::= PrimaryExpressionNotBrace
    |/ SliceExpression
    |/ SelectorExpressionNotBrace Selector",
"SliceExpression ::= 'sl' '(' ')'
    |/ 'sl' '(' Expression ')'
    |/ 'sl' '(' Expression ',' Expression ')'
    |/ 'sl' '(' Expression ',' Expression ',' Expression ')'",
"Selector ::= Arguments
    |/ '.' Identifier
    |/ '.' Identifier Arguments
    |/ '[' Expression ']'",
"Arguments ::= '(' ')'
    |/ '(' ArgumentList ')'",
"ArgumentList ::= NormalArgument
    |/ NamedArgument
    |/ ArgumentList ',' NormalArgument
    |/ ArgumentList ',' NamedArgument",
"NamedArgument ::= Label Expression",
"NormalArgument ::= Expression",
"TypeTest ::= 'is' TypeNotVoid
    |/ 'is' '!' TypeNotVoid",
"TypeCast ::= 'as' TypeNotVoid",
// Statements
"Statements ::= [EMPTY]
    |/ Statements Statement",
"Statement ::= NonLabeledStatement
    |/ Label NonLabeledStatement",
"NonLabeledStatement ::= ExpressionStatement
    |/ LocalVariableDeclaration
    |/ BlockStatement
    |/ IfStatement
    |/ ForStatement
    |/ WhileStatement
    |/ DoStatement
    |/ SwitchStatement
    |/ RethrowStatement
    |/ TryStatement
    |/ BreakStatement
    |/ ContinueStatement
    |/ ReturnStatement",
"ExpressionStatement ::= ExpressionNotBrace ';'",
"LocalVariableDeclaration ::= InitializedVariableDeclaration ';'",
"IfStatement ::= 'if' '(' Expression ')' Statement
    |/ 'if' '(' Expression ')' Statement 'else' Statement",
"ForStatement ::= 'for' '(' ForLoopParts ')' Statement",
"ForLoopParts ::= ForInitializerStatement ExpressionOpt ';' ExpressionListOpt",
"ForInitializerStatement ::= LocalVariableDeclaration
    |/ ExpressionOpt ';'",
"WhileStatement ::= 'while' '(' Expression ')' Statement",
"DoStatement ::= 'do' Statement 'while' '(' Expression ')' ';'",
"SwitchStatement ::= 'switch' '(' Expression ')' '{' DefaultCaseOpt '}'
    |/ 'switch' '(' Expression ')' '{' SwitchCaseList DefaultCaseOpt '}'",
"SwitchCaseList ::= SwitchCase
    |/ SwitchCaseList SwitchCase",
"SwitchCase ::= 'case' Expression ':' Statements",
"DefaultCase ::= 'default' ':' Statements",
"DefaultCaseOpt ::= [EMPTY]
    |/ DefaultCase",
"RethrowStatement ::= 'rethrow' ';'",
"TryStatement ::= 'try' BlockStatement FinallyPart
    |/ 'try' BlockStatement OnPartList
    |/ 'try' BlockStatement OnPartList FinallyPart",
"OnPartList ::= OnPart
    |/ OnPartList OnPart",
"OnPart ::= CatchPart BlockStatement
    |/ 'on' TypeNotVoid BlockStatement
    |/ 'on' TypeNotVoid CatchPart BlockStatement",
"CatchPart ::= 'catch' '(' Identifier ')'
    |/ 'catch' '(' Identifier ',' Identifier ')'",
"FinallyPart ::= 'finally' BlockStatement",
"ReturnStatement ::= 'return' ExpressionOpt ';'",
"Label ::= Identifier ':'",
"BreakStatement ::= 'break' ';'
    |/ 'break' Identifier ';'",
"ContinueStatement ::= 'continue' ';'
    |/ 'continue' Identifier ';'",
// Libraries and Scripts
"LibraryDeclaration ::= LibraryImportList TopLevelDeclarationList",
"TopLevelDeclarationList ::= [EMPTY]
    |/ TopLevelDeclarationList TopLevelDeclaration",
"TopLevelDeclaration ::= ClassDeclaration
    |/ TopFunctionDeclaration
    |/ TopVariableDeclaration",
"LibraryImportList ::= [EMPTY]
    |/ LibraryImportList LibraryImport",
"LibraryImport ::= 'import' Uri ';'
    |/ 'import' Uri 'as' Identifier ';'
    |/ 'import' Uri CombinatorList ';'
    |/ 'import' Uri 'as' Identifier CombinatorList ';'",
"Uri ::= 'STRING_BEGIN_END'",
"CombinatorList ::= Combinator
    |/ CombinatorList Combinator",
"Combinator ::= 'show' IdentifierList
    |/ 'hide' IdentifierList",
"IdentifierList ::= Identifier
    |/ IdentifierList ',' Identifier",
"TopFunctionDeclaration ::= FunctionSignature FunctionBody",
"TopVariableDeclaration ::= 'var' InitializedIdentifierList ';'
    |/ Type InitializedIdentifierList ';'
    |/ 'late' 'var' InitializedIdentifierList ';'
    |/ 'late' Type InitializedIdentifierList ';'
    |/ 'late' 'final' InitializedIdentifierList ';'
    |/ 'late' 'final' Type InitializedIdentifierList ';'",
// Static Types
"Type ::= TypeNotFunction",
"TypeNotVoid ::= TypeNotVoidNotFunction",
"TypeNotFunction ::= 'void'
    |/ TypeNotVoidNotFunction",
"TypeNotVoidNotFunction ::= TypeName",
"TypeName ::= TypeIdentifier
    |/ TypeIdentifier '.' TypeIdentifier",
"TypeArguments ::= '<' TypeList '>'",
"TypeList ::= Type
    |/ TypeList ',' Type",

// Identifier
"Identifier ::= 'IDENTIFIER'
    |/ BUILT_IN_IDENTIFIER
    |/ OTHER_IDENTIFIER",
"TypeIdentifier ::= 'IDENTIFIER'
    |/ OTHER_IDENTIFIER
    |/ 'dynamic'",
"BUILT_IN_IDENTIFIER ::= 'abstract' |/ 'as' |/ 'covariant' |/ 'deferred' |/ 'dynamic' |/ 'export' |/ 'external' |/ 'extension' |/ 'factory' |/ 'Function' |/ 'get' |/ 'implements' |/ 'import' |/ 'interface' |/ 'late' |/ 'library' |/ 'mixin' |/ 'operator' |/ 'part' |/ 'required' |/ 'set' |/ 'static' |/ 'typedef'",
"OTHER_IDENTIFIER ::= 'async' |/ 'hide' |/ 'of' |/ 'on' |/ 'show' |/ 'sync' |/ 'await' |/ 'yield'",

// String
"StringLiteral ::= 'STRING_BEGIN_END'
    |/ 'STRING_BEGIN_MID' StringInterpolation 'STRING_MID_END'",
"StringInterpolation ::= Expression
    |/ StringInterpolation 'STRING_MID_MID' Expression",

// Others
"CommaOpt ::= [EMPTY]
    |/ ','",
];

fn parse_rule(rule: &'static str) -> (&'static str, Vec<ProductionRuleData>) {
    let mut rule_parts = rule.split("::=");
    let name = rule_parts.next().unwrap().trim();
    let alternatives = rule_parts.next().unwrap().trim();

    let alternatives: Vec<ProductionRuleData> = alternatives
        .split("|/")
        .map(|alternative| {
            let symbols = alternative
                .trim()
                .split(" ")
                .map(|symbol| {
                    let symbol = symbol.trim();
                    symbol.trim_matches('\'')
                })
                .collect();

            ProductionRuleData {
                left: name,
                right: symbols,
            }
        })
        .collect();

    (name, alternatives)
}

pub fn get_dart_grammar() -> Vec<ProductionRuleData> {
    let mut rules: Vec<ProductionRuleData> = Vec::new();

    for rule in DART_GRAMMARS.iter() {
        let (_, rule) = parse_rule(rule);
        rules.extend(rule);
    }

    rules
}
