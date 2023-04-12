use crate::parser_generator_lr0::ProductionRuleData;

pub const START_SYMBOL: &'static str = "LibraryDeclaration";

pub const EPSILON: &'static str = "[EMPTY]";
pub const END: &'static str = "[END]";

const DART_GRAMMARS: [&'static str; 37] = [
// Functions
"FunctionSignature ::= 'Identifier' FormalParameterList",
"FunctionBody ::= BlockStatement
    |/ '=>' Expression ';'",
"BlockStatement ::= '{' Statements '}'",
"FormalParameterList ::= '(' ')'
    |/ '(' NormalFormalParameterList ')'",
"NormalFormalParameterList ::= NormalFormalParameter
    |/ NormalFormalParameterList ',' NormalFormalParameter",
"NormalFormalParameter ::= 'Identifier'",
// Expressions
"Expression ::= ConditionalExpression",
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
    |/ PostfixExpression",
"MultiplicativeOperator ::= '*' |/ '/' |/ '%' |/ '~/'",
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
"Statement ::= ExpressionStatement",
"ExpressionStatement ::= Expression ';'",
// Libraries and Scripts
"LibraryDeclaration ::= TopLevelDeclarationList",
"TopLevelDeclarationList ::= [EMPTY]
    |/ TopLevelDeclarationList TopLevelDeclaration",
"TopLevelDeclaration ::= TopFunctionDeclaration",
"TopFunctionDeclaration ::= FunctionSignature FunctionBody",
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
