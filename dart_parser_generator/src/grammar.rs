use crate::parser_generator_lr0::ProductionRuleData;

pub const START_SYMBOL: &'static str = "LibraryDeclaration";

pub const EPSILON: &'static str = "[EMPTY]";
pub const END: &'static str = "[END]";

const DART_GRAMMARS: [&'static str; 74] = [
// Variables
"InitializedVariableDeclaration ::= DeclaredIdentifier
    |/ DeclaredIdentifier '=' Expression
    |/ InitializedVariableDeclaration ',' InitializedIdentifier",
"InitializedIdentifier ::= 'Identifier'
    |/ 'Identifier' '=' Expression",
"InitializedIdentifierList ::= InitializedIdentifier
    |/ InitializedIdentifierList ',' InitializedIdentifier",
// Functions
"FunctionSignature ::= 'Identifier' FormalParameterList
    |/ Type 'Identifier' FormalParameterList",
"FunctionBody ::= BlockStatement
    |/ '=>' Expression ';'",
"BlockStatement ::= '{' Statements '}'",
"FormalParameterList ::= '(' ')'
    |/ '(' NormalFormalParameterList ')'",
"NormalFormalParameterList ::= NormalFormalParameter
    |/ NormalFormalParameterList ',' NormalFormalParameter",
"NormalFormalParameter ::= 'Identifier'",
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
// Expressions
"Expression ::= ConditionalExpression",
"ExpressionOpt ::= [EMPTY]
    |/ Expression",
"ExpressionList ::= Expression
    |/ ExpressionList ',' Expression",
"ExpressionListOpt ::= [EMPTY]
    |/ ExpressionList",
"PrimaryExpression ::= '(' Expression ')'
    |/ 'Boolean'
    |/ 'Number'
    |/ StringLiteralList
    |/ 'Identifier'",
"StringLiteralList ::= 'String'
    |/ StringLiteralList 'String'",
"ConditionalExpression ::= IfNullExpression
    |/ IfNullExpression '?' Expression : Expression",
"IfNullExpression ::= LogicalOrExpression
    |/ IfNullExpression '??' LogicalOrExpression",
"LogicalOrExpression ::= LogicalAndExpression
    |/ LogicalOrExpression '||' LogicalAndExpression",
"LogicalAndExpression ::= EqualityExpression
    |/ LogicalAndExpression '&&' EqualityExpression",
"EqualityExpression ::= RelationalExpression
    |/ RelationalExpression EqualityOperator RelationalExpression",
"EqualityOperator ::= '==' |/ '!='",
"RelationalExpression ::= BitwiseOrExpression
    |/ BitwiseOrExpression RelationalOperator BitwiseOrExpression",
"RelationalOperator ::= '<' |/ '>' |/ '<=' |/ '>='",
"BitwiseOrExpression ::= BitwiseXorExpression
    |/ BitwiseOrExpression '|' BitwiseXorExpression",
"BitwiseXorExpression ::= BitwiseAndExpression
    |/ BitwiseXorExpression '^' BitwiseAndExpression",
"BitwiseAndExpression ::= ShiftExpression
    |/ BitwiseAndExpression '&' ShiftExpression",
"ShiftExpression ::= AdditiveExpression
    |/ ShiftExpression ShiftOperator AdditiveExpression",
"ShiftOperator ::= '<<' |/ '>>'",
"AdditiveExpression ::= AdditiveExpression '+' MultiplicativeExpression
    |/ AdditiveExpression '-' MultiplicativeExpression
    |/ MultiplicativeExpression",
"MultiplicativeExpression ::= MultiplicativeExpression MultiplicativeOperator PrimaryExpression
    |/ UnaryExpression",
"MultiplicativeOperator ::= '*' |/ '/' |/ '%' |/ '~/'",
"UnaryExpression ::= PostfixExpression
    |/ PrefixOperator UnaryExpression
    |/ IncrementOperator UnaryExpression",
"PrefixOperator ::= '!' |/ '-' |/ '~'",
"IncrementOperator ::= '++' |/ '--'",
"PostfixExpression ::= PrimaryExpression
    |/ PostfixExpression Selector",
"Selector ::= Arguments",
"Arguments ::= '(' ')'
    |/ '(' ArgumentList ')'",
"ArgumentList ::= NormalArgument
    |/ ArgumentList ',' NormalArgument",
"NormalArgument ::= Expression",
// Statements
"Statements ::= [EMPTY]
    |/ Statements Statement",
"Statement ::= ExpressionStatement
    |/ LocalVariableDeclaration
    |/ BlockStatement
    |/ IfStatement
    |/ ForStatement
    |/ WhileStatement
    |/ DoStatement
    |/ SwitchStatement
    |/ RethrowStatement
    |/ TryStatement
    |/ ReturnStatement",
"ExpressionStatement ::= Expression ';'",
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
// Libraries and Scripts
"LibraryDeclaration ::= TopLevelDeclarationList",
"TopLevelDeclarationList ::= [EMPTY]
    |/ TopLevelDeclarationList TopLevelDeclaration",
"TopLevelDeclaration ::= TopFunctionDeclaration
    |/ TopVariableDeclaration",
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
"TypeName ::= 'Identifier'",
"TypeArguments ::= '<' TypeList '>'",
"TypeList ::= Type
    |/ TypeList ',' Type",
];

// const DART_GRAMMARS: [&'static str; 3] = [
// "ArgumentList ::= '(' ExpressionList ')'",
// "ExpressionList ::= [EMPTY]
//     |/ Expression
//     |/ ExpressionList ',' Expression",
// "Expression ::= 'Number'",
// ];

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
