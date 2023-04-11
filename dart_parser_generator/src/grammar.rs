use crate::parser_generator_lr0::ProductionRuleData;

pub const START_SYMBOL: &'static str = "AdditiveExpression";

pub const EPSILON: &'static str = "[EMPTY]";
pub const END: &'static str = "[END]";

const DART_GRAMMARS: [&'static str; 8] = [
"AdditiveExpression ::= AdditiveExpression '+' MultiplicativeExpression
    |/ AdditiveExpression '-' MultiplicativeExpression
    |/ MultiplicativeExpression",
"MultiplicativeExpression ::= MultiplicativeExpression '*' PrimaryExpression
    |/ MultiplicativeExpression '/' PrimaryExpression
    |/ PostfixExpression",
"PostfixExpression ::= PrimaryExpression
    |/ PostfixExpression Selector",
"Selector ::= Arguments",
"Arguments ::= '(' ')'
    |/ '(' ArgumentList ')'",
"ArgumentList ::= NormalArgument
    |/ ArgumentList ',' NormalArgument",
"NormalArgument ::= AdditiveExpression",
"PrimaryExpression ::= '(' AdditiveExpression ')'
    |/ 'Number'
    |/ 'Identifier'",
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
