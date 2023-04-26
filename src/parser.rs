use anyhow::{bail, ensure, Result};
use dart_parser_generator::{
    grammar::EPSILON,
    parser_generator::{SerializableRule, State, TransitionData, TransitionMap},
};

use crate::tokenizer::Token;

use self::parse_library::parse_library;
use self::{node::*, node_internal::NodeInternal};

pub mod node;
mod node_internal;
mod parse_class;
mod parse_expression;
mod parse_functions;
mod parse_identifier;
mod parse_library;
mod parse_literal;
mod parse_selector;
mod parse_statement;
mod parse_type;
mod parse_variables;
mod util;

pub fn parse<'input>(
    input: Vec<Token<'input>>,
    transition_map: TransitionMap,
) -> Result<LibraryDeclaration> {
    let internal_node = parse_internally(input, transition_map)?;
    parse_library(&internal_node)
}

// 初期状態
// input: token, token, token, token, ...
// stack: (I0, _) ->
// node_stack: ->
// parse_index = 0
// accepted = false
//
// 1. stackの一番上のstateとparse_index位置のinputのtokenからtrnasitionを取得
// 2. transitionが存在しなければError
// 3. transitionがShiftの場合:
//    3.1. stackに遷移先のstateと現在のtokenを積む
//    3.2. node_stackに現在のtokenをnodeとして積む
//    3.3. parse_indexを1進める
// 4. transitionがReduceの場合:
//    4.1. child_sizeを決定(ruleの右辺がεの場合は0、それ以外はその長さ)
//    4.2. child_sizeだけstackとnode_stackをpopし、node_stackの方はchildrenとする
//    4.3. 還元したruleのnodeを作成
//    4.4. 次のtransition判定は還元後のruleを使うのでここでshift情報をstackに積む(reduce/acceptは原理上起こらない)
//    4.5. nodeをnode_stackに積む
// 5. transitionがAcceptの場合:
//    5.1. acceptをtrueにして終了
fn parse_internally<'input>(
    input: Vec<Token<'input>>,
    transition_map: TransitionMap,
) -> Result<NodeInternal<'input>> {
    let mut stack: Vec<String> = Vec::new();
    let node_stack: Vec<NodeInternal> = Vec::new();
    let parse_index = 0;

    stack.push(String::from("I0"));

    build_internal_node(&input, &transition_map, stack, node_stack, parse_index)
}

fn build_internal_node<'input>(
    input: &Vec<Token<'input>>,
    transition_map: &TransitionMap,
    mut stack: Vec<State>,
    mut node_stack: Vec<NodeInternal<'input>>,
    mut parse_index: usize,
) -> Result<NodeInternal<'input>> {
    let mut accepted = false;
    while parse_index < input.len() || !stack.is_empty() {
        let transition = transition_map
            .transitions
            .get(&(stack.last().unwrap().clone(), input[parse_index].kind_str()));

        ensure!(
            transition.is_some(),
            "No Transition Error: {:?}, {}",
            input[parse_index],
            stack.last().unwrap()
        );

        let transition = transition.unwrap();
        match transition {
            TransitionData::Shift { target } => {
                stack.push(target.clone());
                node_stack.push(NodeInternal {
                    rule_name: input[parse_index].kind_str(),
                    token: Some(input[parse_index].clone()),
                    children: Vec::new(),
                });
                parse_index += 1;
            }
            TransitionData::Reduce { rule } => {
                reduce_rule(
                    &mut stack,
                    &mut node_stack,
                    &transition_map,
                    input[parse_index].clone(),
                    rule,
                )?;
            }
            TransitionData::Accept => {
                accepted = true;
                break;
            }
            TransitionData::ReduceReduceConflict { rules } => {
                // 1つずつ試してみて、構文エラーが起きたら次のルールを試す
                for selected_rule in rules {
                    let mut copied_stack = stack.clone();
                    let mut copied_node_stack = node_stack.clone();
                    let token = input[parse_index].clone();

                    let result = reduce_rule(
                        &mut copied_stack,
                        &mut copied_node_stack,
                        &transition_map,
                        token,
                        selected_rule,
                    );
                    if result.is_err() {
                        continue;
                    }

                    let result = build_internal_node(
                        input,
                        transition_map,
                        copied_stack,
                        copied_node_stack,
                        parse_index,
                    );
                    match result {
                        Ok(node) => {
                            println!("Reduce-Reduce Conflict resolved: {:?}", selected_rule);
                            return Ok(node);
                        }
                        Err(_) => {
                            continue;
                        }
                    }
                }

                bail!("Reduce-Reduce Conflict");
            }
        }
    }

    if accepted {
        Ok(node_stack.pop().unwrap())
    } else {
        bail!("Parse Error")
    }
}

fn reduce_rule<'input>(
    stack: &mut Vec<State>,
    node_stack: &mut Vec<NodeInternal<'input>>,
    transition_map: &TransitionMap,
    token: Token,
    rule: &SerializableRule,
) -> Result<()> {
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
    node_stack.push(new_node);

    let next_transition = transition_map
        .transitions
        .get(&(stack.last().unwrap().clone(), rule.left.to_string()));

    ensure!(
        next_transition.is_some(),
        "No Transition Error: {:?}, {}",
        token,
        stack.last().unwrap()
    );

    if let TransitionData::Shift { target } = next_transition.unwrap() {
        stack.push(target.clone());
        Ok(())
    } else {
        bail!(
            "No Transition Error: {:?}, {}",
            token,
            stack.last().unwrap()
        );
    }
}
