use dart_parser_generator::{
    grammar::EPSILON,
    parser_generator::{TransitionKind, TransitionMap},
};
use std::error::Error;

use crate::tokenizer::Token;

use self::parse_library::parse_library;
use self::{node::*, node_internal::NodeInternal};

pub mod node;
mod node_internal;
mod parse_expression;
mod parse_functions;
mod parse_identifier;
mod parse_library;
mod parse_selector;
mod parse_statement;
mod parse_type;
mod parse_variables;
mod util;

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
            println!(
                "No Transition Error: {:?}, {}",
                input[parse_index],
                stack.last().unwrap().0
            );
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
