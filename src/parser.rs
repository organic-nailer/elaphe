use dart_parser_generator::{
    grammar::EPSILON,
    parser_generator::{TransitionKind, TransitionMap},
};
use std::{error::Error, fmt};

use crate::tokenizer::{Token, TokenKind};

use self::node::*;

pub mod node;

pub fn parse<'input>(
    input: Vec<Token<'input>>,
    transition_map: TransitionMap,
) -> Result<LibraryDeclaration, Box<dyn Error>> {
    let internal_node = parse_internally(input, transition_map);
    println!("accepted from the parser");

    parse_library(&internal_node)
}

fn parse_internally(input: Vec<Token>, transition_map: TransitionMap) -> NodeInternal {
    let mut stack: Vec<(String, String)> = Vec::new();
    let mut node_stack: Vec<NodeInternal> = Vec::new();
    let mut parse_index = 0;
    let mut accepted = false;

    // Stackの先頭のTokenは使わないのでなんでもよい
    stack.push((String::from("I0"), "".to_string()));
    while parse_index < input.len() || !stack.is_empty() {
        // println!("stack: {:?}, index: {}", stack, parse_index);
        let transition = transition_map.transitions.get(&(
            stack.last().unwrap().0.clone(),
            input[parse_index].kind_str(),
        ));

        if transition.is_none() {
            println!("No Transition Error: {:?}", input[parse_index]);
            break;
        }
        let transition = transition.unwrap();
        match transition.kind {
            TransitionKind::Shift => {
                stack.push((
                    transition.target.clone().unwrap(),
                    input[parse_index].kind_str(),
                ));
                node_stack.push(NodeInternal {
                    rule_name: input[parse_index].kind_str(),
                    token: Some(input[parse_index].clone()),
                    children: Vec::new(),
                });
                parse_index += 1;
            }
            TransitionKind::Reduce => {
                let rule = transition.rule.clone().unwrap();
                let mut children = Vec::new();
                let child_size = if rule.right.len() == 1 && rule.right[0] == EPSILON {
                    0
                } else {
                    rule.right.len()
                };
                for _ in 0..child_size {
                    stack.pop();
                    children.push(node_stack.pop().unwrap());
                }
                children.reverse();
                let new_node = NodeInternal {
                    rule_name: rule.left.to_string(),
                    token: None,
                    children,
                };

                let next_transition = transition_map
                    .transitions
                    .get(&(stack.last().unwrap().0.clone(), rule.left.to_string()));
                if next_transition.is_none() {
                    println!("Shift-Reduce Conflict: {:?}", rule);
                    break;
                }
                let next_transition = next_transition.unwrap();
                stack.push((
                    next_transition.target.clone().unwrap(),
                    rule.left.to_string(),
                ));

                node_stack.push(new_node);
            }
            TransitionKind::Accept => {
                accepted = true;
                break;
            }
        }
    }

    if accepted {
        node_stack.pop().unwrap()
    } else {
        panic!("Parse Error");
    }
}

pub struct NodeInternal<'input> {
    rule_name: String,
    children: Vec<NodeInternal<'input>>,
    token: Option<Token<'input>>,
}

fn parse_library<'input>(
    node: &NodeInternal<'input>,
) -> Result<LibraryDeclaration<'input>, Box<dyn Error>> {
    Ok(LibraryDeclaration {
        top_level_declaration_list: parse_top_level_declaration_list(&node.children[0])?,
    })
}

fn parse_top_level_declaration_list<'input>(
    node: &NodeInternal<'input>,
) -> Result<Vec<Box<NodeStatement<'input>>>, Box<dyn Error>> {
    if node.rule_name == "TopLevelDeclarationList" {
        if node.children.len() == 0 {
            return Ok(vec![]);
        } else {
            return flatten(
                parse_top_level_declaration_list(&node.children[0]),
                Box::new(parse_top_level_declaration(&node.children[1])?),
            );
        }
    }

    Err("Parse Error parse_top_level_declaration_list".into())
}

fn parse_top_level_declaration<'input>(
    node: &NodeInternal<'input>,
) -> Result<NodeStatement<'input>, Box<dyn Error>> {
    if node.rule_name == "TopLevelDeclaration" {
        return parse_top_function_declaration(&node.children[0]);
    }

    Err("Parse Error argument_list".into())
}

fn parse_top_function_declaration<'input>(
    node: &NodeInternal<'input>,
) -> Result<NodeStatement<'input>, Box<dyn Error>> {
    if node.rule_name == "TopFunctionDeclaration" {
        return Ok(NodeStatement::FunctionDeclaration {
            signature: parse_function_signature(&node.children[0])?,
            body: Box::new(parse_function_body(&node.children[1])?),
        });
    }

    Err("Parse Error argument_list".into())
}

fn parse_formal_parameter_list<'input>(
    node: &NodeInternal<'input>,
) -> Result<Vec<Identifier<'input>>, Box<dyn Error>> {
    if node.rule_name == "FormalParameterList" {
        if node.children.len() == 2 {
            return Ok(vec![]);
        } else {
            return parse_normal_formal_parameter_list(&node.children[1]);
        }
    }

    Err("Parse Error parse_formal_parameter_list".into())
}

fn parse_normal_formal_parameter_list<'input>(
    node: &NodeInternal<'input>,
) -> Result<Vec<Identifier<'input>>, Box<dyn Error>> {
    if node.rule_name == "NormalFormalParameterList" {
        if node.children.len() == 1 {
            return Ok(vec![parse_normal_formal_parameter(&node.children[0])?]);
        } else {
            return flatten(
                parse_normal_formal_parameter_list(&node.children[0]),
                parse_normal_formal_parameter(&node.children[2])?,
            );
        }
    }

    Err("Parse Error parse_normal_formal_parameter_list".into())
}

fn parse_normal_formal_parameter<'input>(
    node: &NodeInternal<'input>,
) -> Result<Identifier<'input>, Box<dyn Error>> {
    if node.rule_name == "NormalFormalParameter" {
        return Ok(parse_identifier(&node.children[0])?);
    }

    Err("Parse Error parse_normal_formal_parameter".into())
}

fn parse_function_signature<'input>(
    node: &NodeInternal<'input>,
) -> Result<FunctionSignature<'input>, Box<dyn Error>> {
    if node.rule_name == "FunctionSignature" {
        return Ok(FunctionSignature {
            name: parse_identifier(&node.children[0])?,
            param: parse_formal_parameter_list(&node.children[1])?,
        });
    }

    Err("Parse Error parse_function_signature".into())
}

fn parse_function_body<'input>(
    node: &NodeInternal<'input>,
) -> Result<NodeStatement<'input>, Box<dyn Error>> {
    if node.rule_name == "FunctionBody" {
        if node.children.len() == 1 {
            return parse_statement(&node.children[0]);
        } else {
            return parse_statement(&node.children[1]);
        }
    }

    Err("Parse Error parse_function_body".into())
}

fn parse_expression<'input>(
    node: &NodeInternal<'input>,
) -> Result<NodeExpression<'input>, Box<dyn Error>> {
    match node.rule_name.as_str() {
        "AdditiveExpression" | "MultiplicativeExpression" => {
            if node.children.len() == 1 {
                parse_expression(&node.children[0])
            } else {
                let left = parse_expression(&node.children[0])?;
                let right = parse_expression(&node.children[2])?;
                Ok(NodeExpression::Binary {
                    left: Box::new(left),
                    right: Box::new(right),
                    operator: &node.children[1].token.clone().unwrap().str,
                })
            }
        }
        "PostfixExpression" => {
            if node.children.len() == 1 {
                parse_expression(&node.children[0])
            } else {
                let left = parse_expression(&node.children[0])?;
                Ok(NodeExpression::Selector {
                    left: Box::new(left),
                    operator: parse_selector(&node.children[1])?,
                })
            }
        }
        "PrimaryExpression" => {
            if node.children.len() == 1 {
                parse_expression(&node.children[0])
            } else {
                parse_expression(&node.children[1])
            }
        }
        "Boolean" => Ok(NodeExpression::BooleanLiteral {
            value: node.token.clone().unwrap().str,
        }),
        "Number" => Ok(NodeExpression::NumericLiteral {
            value: node.token.clone().unwrap().str,
        }),
        "Identifier" => Ok(NodeExpression::Identifier {
            identifier: parse_identifier(node)?,
        }),
        v => Err(format!("Parse Error: {} is not valid rule in expression", v).into()),
    }
}

fn parse_statement<'input>(
    node: &NodeInternal<'input>,
) -> Result<NodeStatement<'input>, Box<dyn Error>> {
    match node.rule_name.as_str() {
        "Statement" => parse_statement(&node.children[0]),
        "BlockStatement" => Ok(NodeStatement::BlockStatement {
            statements: parse_statement_list(&node.children[1])?,
        }),
        "ExpressionStatement" => Ok(NodeStatement::ExpressionStatement {
            expr: Box::new(parse_expression(&node.children[0])?),
        }),
        v => Err(format!("Parse Error: {} is not valid rule in statement", v).into()),
    }
}

fn parse_statement_list<'input>(
    node: &NodeInternal<'input>,
) -> Result<Vec<NodeStatement<'input>>, Box<dyn Error>> {
    if node.rule_name == "Statements" {
        if node.children.len() == 0 {
            return Ok(vec![]);
        } else {
            return flatten(
                parse_statement_list(&node.children[0]),
                parse_statement(&node.children[1])?,
            );
        }
    }

    Err("Parse Error parse_statement_list".into())
}

fn flatten<T>(left: Result<Vec<T>, Box<dyn Error>>, right: T) -> Result<Vec<T>, Box<dyn Error>> {
    let mut flt = left?;
    flt.push(right);
    Ok(flt)
}

fn parse_selector<'input>(node: &NodeInternal<'input>) -> Result<Selector<'input>, Box<dyn Error>> {
    if node.rule_name != "Selector" {
        return Err("Parse Error in selector".into());
    }
    let node = &node.children[0];
    match node.rule_name.as_str() {
        "Arguments" => {
            if node.children.len() == 2 {
                Ok(Selector::Args { args: Vec::new() })
            } else {
                Ok(Selector::Args {
                    args: parse_argument_list(&node.children[1])?,
                })
            }
        }
        v => Err(format!("Parse Error: {} is not valid rule in selector", v).into()),
    }
}

fn parse_normal_argument<'input>(
    node: &NodeInternal<'input>,
) -> Result<CallParameter<'input>, Box<dyn Error>> {
    if node.rule_name == "NormalArgument" {
        return Ok(CallParameter {
            identifier: None,
            expr: Box::new(parse_expression(&node.children[0])?),
        });
    }

    Err("Parse Error in normal_argument".into())
}

fn parse_argument_list<'input>(
    node: &NodeInternal<'input>,
) -> Result<Vec<CallParameter<'input>>, Box<dyn Error>> {
    if node.rule_name == "ArgumentList" {
        if node.children.len() == 1 {
            return Ok(vec![parse_normal_argument(&node.children[0])?]);
        } else {
            return flatten(
                parse_argument_list(&node.children[0]),
                parse_normal_argument(&node.children[2])?,
            );
        }
    }

    Err("Parse Error argument_list".into())
}

fn parse_identifier<'input>(
    node: &NodeInternal<'input>,
) -> Result<Identifier<'input>, Box<dyn Error>> {
    if node.rule_name == "Identifier" {
        return Ok(Identifier {
            value: node.token.clone().unwrap().str,
        });
    }

    Err("Parse Error in identifier".into())
}
