use std::{error::Error, vec};

use super::{
    node::{DefaultCase, Identifier, NodeStatement, SwitchCase, TryCatchPart, TryOnPart},
    node_internal::NodeInternal,
    parse_expression::{parse_expression, parse_expression_list_opt, parse_expression_opt},
    parse_identifier::parse_identifier,
    parse_type::parse_type,
    parse_variables::parse_initialized_variable_declaration,
    util::{flatten, gen_error},
};

pub fn parse_statement<'input>(
    node: &NodeInternal<'input>,
) -> Result<NodeStatement<'input>, Box<dyn Error>> {
    if node.rule_name == "Statement" {
        if node.children.len() == 1 {
            return parse_non_labeled_statement(&node.children[0]);
        } else if node.children.len() == 2 {
            return Ok(NodeStatement::Labeled {
                label: parse_label(&node.children[0])?,
                stmt: Box::new(parse_non_labeled_statement(&node.children[1])?),
            });
        }
    }

    Err(gen_error("parse_statement", &node.rule_name))
}

pub fn parse_non_labeled_statement<'input>(
    node: &NodeInternal<'input>,
) -> Result<NodeStatement<'input>, Box<dyn Error>> {
    if node.rule_name != "NonLabeledStatement" {
        return Err(gen_error("parse_non_labeled_statement", &node.rule_name));
    }
    let node = &node.children[0];
    match node.rule_name.as_str() {
        "LocalVariableDeclaration" => parse_local_variable_declaration(node),
        "BlockStatement" => parse_block_statement(node),
        "ExpressionStatement" => Ok(NodeStatement::Expression {
            expr: Box::new(parse_expression(&node.children[0])?),
        }),
        "IfStatement" => parse_if_statement(node),
        "ForStatement" => parse_for_statement(node),
        "WhileStatement" => parse_while_statement(node),
        "DoStatement" => parse_do_statement(node),
        "SwitchStatement" => parse_switch_statement(node),
        "RethrowStatement" => parse_rethrow_statement(node),
        "TryStatement" => parse_try_statement(node),
        "ReturnStatement" => parse_return_statement(node),
        "BreakStatement" => parse_break_statement(node),
        "ContinueStatement" => parse_continue_statement(node),
        v => Err(gen_error("parse_non_labeled_statement", v)),
    }
}

pub fn parse_block_statement<'input>(
    node: &NodeInternal<'input>,
) -> Result<NodeStatement<'input>, Box<dyn Error>> {
    if node.rule_name == "BlockStatement" {
        return Ok(NodeStatement::Block {
            statements: parse_statement_list(&node.children[1])?,
        });
    }

    Err(gen_error("parse_block_statement", &node.rule_name))
}

pub fn parse_statement_list<'input>(
    node: &NodeInternal<'input>,
) -> Result<Vec<Box<NodeStatement<'input>>>, Box<dyn Error>> {
    if node.rule_name == "Statements" {
        if node.children.len() == 0 {
            return Ok(vec![]);
        } else {
            return flatten(
                parse_statement_list(&node.children[0]),
                Box::new(parse_statement(&node.children[1])?),
            );
        }
    }

    Err(gen_error("parse_statement_list", &node.rule_name))
}

fn parse_local_variable_declaration<'input>(
    node: &NodeInternal<'input>,
) -> Result<NodeStatement<'input>, Box<dyn Error>> {
    if node.rule_name == "LocalVariableDeclaration" {
        return Ok(NodeStatement::VariableDeclarationList {
            decl_list: parse_initialized_variable_declaration(&node.children[0])?,
        });
    }

    Err(gen_error(
        "parse_local_variable_declaration",
        &node.rule_name,
    ))
}

fn parse_if_statement<'input>(
    node: &NodeInternal<'input>,
) -> Result<NodeStatement<'input>, Box<dyn Error>> {
    if node.rule_name == "IfStatement" {
        if node.children.len() == 5 {
            return Ok(NodeStatement::If {
                condition: Box::new(parse_expression(&node.children[2])?),
                if_true_stmt: Box::new(parse_statement(&node.children[4])?),
                if_false_stmt: None,
            });
        } else if node.children.len() == 7 {
            return Ok(NodeStatement::If {
                condition: Box::new(parse_expression(&node.children[2])?),
                if_true_stmt: Box::new(parse_statement(&node.children[4])?),
                if_false_stmt: Some(Box::new(parse_statement(&node.children[6])?)),
            });
        }
    }

    Err(gen_error("parse_if_statement", &node.rule_name))
}

fn parse_for_statement<'input>(
    node: &NodeInternal<'input>,
) -> Result<NodeStatement<'input>, Box<dyn Error>> {
    if node.rule_name == "ForStatement" {
        let parts_node = &node.children[2];
        let parts_init_node = &parts_node.children[0];
        let init = if parts_init_node.children.len() == 1 {
            if parts_init_node.children[0].rule_name == "LocalVariableDeclaration" {
                Some(Box::new(parse_local_variable_declaration(
                    &parts_init_node.children[0],
                )?))
            } else {
                None
            }
        } else {
            Some(Box::new(NodeStatement::Expression {
                expr: Box::new(parse_expression(&parts_init_node.children[0])?),
            }))
        };
        return Ok(NodeStatement::For {
            init,
            condition: parse_expression_opt(&parts_node.children[1])?,
            update: parse_expression_list_opt(&parts_node.children[3])?,
            stmt: Box::new(parse_statement(&node.children[4])?),
        });
    }

    Err(gen_error("parse_for_statement", &node.rule_name))
}

fn parse_while_statement<'input>(
    node: &NodeInternal<'input>,
) -> Result<NodeStatement<'input>, Box<dyn Error>> {
    if node.rule_name == "WhileStatement" {
        return Ok(NodeStatement::While {
            condition: Box::new(parse_expression(&node.children[2])?),
            stmt: Box::new(parse_statement(&node.children[4])?),
        });
    }

    Err(gen_error("parse_while_statement", &node.rule_name))
}

fn parse_do_statement<'input>(
    node: &NodeInternal<'input>,
) -> Result<NodeStatement<'input>, Box<dyn Error>> {
    if node.rule_name == "DoStatement" {
        return Ok(NodeStatement::Do {
            stmt: Box::new(parse_statement(&node.children[1])?),
            condition: Box::new(parse_expression(&node.children[4])?),
        });
    }

    Err(gen_error("parse_do_statement", &node.rule_name))
}

fn parse_switch_statement<'input>(
    node: &NodeInternal<'input>,
) -> Result<NodeStatement<'input>, Box<dyn Error>> {
    if node.rule_name == "SwitchStatement" {
        if node.children.len() == 7 {
            return Ok(NodeStatement::Switch {
                expr: Box::new(parse_expression(&node.children[2])?),
                case_list: vec![],
                default_case: parse_default_case_opt(&node.children[5])?,
            });
        } else {
            return Ok(NodeStatement::Switch {
                expr: Box::new(parse_expression(&node.children[2])?),
                case_list: parse_switch_case_list(&node.children[5])?,
                default_case: parse_default_case_opt(&node.children[6])?,
            });
        }
    }

    Err(gen_error("parse_switch_statement", &node.rule_name))
}

fn parse_switch_case_list<'input>(
    node: &NodeInternal<'input>,
) -> Result<Vec<SwitchCase<'input>>, Box<dyn Error>> {
    if node.rule_name == "SwitchCaseList" {
        if node.children.len() == 1 {
            return Ok(vec![parse_switch_case(&node.children[0])?]);
        } else {
            return flatten(
                parse_switch_case_list(&node.children[0]),
                parse_switch_case(&node.children[1])?,
            );
        }
    }

    Err(gen_error("parse_switch_case_list", &node.rule_name))
}

fn parse_switch_case<'input>(
    node: &NodeInternal<'input>,
) -> Result<SwitchCase<'input>, Box<dyn Error>> {
    if node.rule_name == "SwitchCase" {
        return Ok(SwitchCase {
            label_list: vec![],
            expr: Box::new(parse_expression(&node.children[1])?),
            stmt_list: parse_statement_list(&node.children[3])?,
        });
    }

    Err(gen_error("parse_switch_case", &node.rule_name))
}

fn parse_default_case<'input>(
    node: &NodeInternal<'input>,
) -> Result<DefaultCase<'input>, Box<dyn Error>> {
    if node.rule_name == "DefaultCase" {
        return Ok(DefaultCase {
            label_list: vec![],
            stmt_list: parse_statement_list(&node.children[2])?,
        });
    }

    Err(gen_error("parse_default_case", &node.rule_name))
}

fn parse_default_case_opt<'input>(
    node: &NodeInternal<'input>,
) -> Result<Option<DefaultCase<'input>>, Box<dyn Error>> {
    if node.rule_name == "DefaultCaseOpt" {
        if node.children.len() == 0 {
            return Ok(None);
        } else {
            return Ok(Some(parse_default_case(&node.children[0])?));
        }
    }

    Err(gen_error("parse_default_case_opt", &node.rule_name))
}

fn parse_rethrow_statement<'input>(
    node: &NodeInternal<'input>,
) -> Result<NodeStatement<'input>, Box<dyn Error>> {
    if node.rule_name == "RethrowStatement" {
        return Ok(NodeStatement::Rethrow);
    }

    Err(gen_error("parse_rethrow_statement", &node.rule_name))
}

fn parse_try_statement<'input>(
    node: &NodeInternal<'input>,
) -> Result<NodeStatement<'input>, Box<dyn Error>> {
    if node.rule_name == "TryStatement" {
        if node.children.len() == 3 {
            if node.children[2].rule_name == "FinallyPart" {
                return Ok(NodeStatement::TryFinally {
                    block_try: Box::new(parse_block_statement(&node.children[1])?),
                    block_finally: Box::new(parse_finally_part(&node.children[2])?),
                });
            } else {
                return Ok(NodeStatement::TryOn {
                    block_try: Box::new(parse_block_statement(&node.children[1])?),
                    on_part_list: parse_on_part_list(&node.children[2])?,
                });
            }
        } else {
            return Ok(NodeStatement::TryFinally {
                block_try: Box::new(NodeStatement::TryOn {
                    block_try: Box::new(parse_block_statement(&node.children[1])?),
                    on_part_list: parse_on_part_list(&node.children[2])?,
                }),
                block_finally: Box::new(parse_finally_part(&node.children[3])?),
            });
        }
    }

    Err(gen_error("parse_try_statement", &node.rule_name))
}

fn parse_on_part<'input>(node: &NodeInternal<'input>) -> Result<TryOnPart<'input>, Box<dyn Error>> {
    if node.rule_name == "OnPart" {
        if node.children.len() == 2 {
            return Ok(TryOnPart {
                catch_part: Some(parse_catch_part(&node.children[0])?),
                exc_type: None,
                block: Box::new(parse_block_statement(&node.children[1])?),
            });
        } else if node.children.len() == 3 {
            return Ok(TryOnPart {
                catch_part: None,
                exc_type: Some(parse_type(&node.children[1])?),
                block: Box::new(parse_block_statement(&node.children[2])?),
            });
        } else {
            return Ok(TryOnPart {
                catch_part: Some(parse_catch_part(&node.children[2])?),
                exc_type: Some(parse_type(&node.children[1])?),
                block: Box::new(parse_block_statement(&node.children[3])?),
            });
        }
    }

    Err(gen_error("parse_on_part", &node.rule_name))
}

fn parse_on_part_list<'input>(
    node: &NodeInternal<'input>,
) -> Result<Vec<TryOnPart<'input>>, Box<dyn Error>> {
    if node.rule_name == "OnPartList" {
        if node.children.len() == 1 {
            return Ok(vec![parse_on_part(&node.children[0])?]);
        } else {
            return flatten(
                parse_on_part_list(&node.children[0]),
                parse_on_part(&node.children[1])?,
            );
        }
    }

    Err(gen_error("parse_on_part_list", &node.rule_name))
}

fn parse_catch_part<'input>(
    node: &NodeInternal<'input>,
) -> Result<TryCatchPart<'input>, Box<dyn Error>> {
    if node.rule_name == "CatchPart" {
        if node.children.len() == 4 {
            return Ok(TryCatchPart {
                id_error: parse_identifier(&node.children[2])?,
                id_trace: None,
            });
        } else {
            return Ok(TryCatchPart {
                id_error: parse_identifier(&node.children[2])?,
                id_trace: Some(parse_identifier(&node.children[4])?),
            });
        }
    }

    Err(gen_error("parse_catch_part", &node.rule_name))
}

fn parse_finally_part<'input>(
    node: &NodeInternal<'input>,
) -> Result<NodeStatement<'input>, Box<dyn Error>> {
    if node.rule_name == "FinallyPart" {
        return Ok(parse_block_statement(&node.children[1])?);
    }

    Err(gen_error("parse_finally_part", &node.rule_name))
}

fn parse_return_statement<'input>(
    node: &NodeInternal<'input>,
) -> Result<NodeStatement<'input>, Box<dyn Error>> {
    if node.rule_name == "ReturnStatement" {
        return Ok(NodeStatement::Return {
            value: parse_expression_opt(&node.children[1])?,
        });
    }

    Err(gen_error("parse_return_statement", &node.rule_name))
}

fn parse_break_statement<'input>(
    node: &NodeInternal<'input>,
) -> Result<NodeStatement<'input>, Box<dyn Error>> {
    if node.rule_name == "BreakStatement" {
        if node.children.len() == 2 {
            return Ok(NodeStatement::Break { label: None });
        } else {
            return Ok(NodeStatement::Break {
                label: Some(parse_identifier(&node.children[1])?),
            });
        }
    }

    Err(gen_error("parse_break_statement", &node.rule_name))
}

fn parse_continue_statement<'input>(
    node: &NodeInternal<'input>,
) -> Result<NodeStatement<'input>, Box<dyn Error>> {
    if node.rule_name == "ContinueStatement" {
        if node.children.len() == 2 {
            return Ok(NodeStatement::Continue { label: None });
        } else {
            return Ok(NodeStatement::Continue {
                label: Some(parse_identifier(&node.children[1])?),
            });
        }
    }

    Err(gen_error("parse_continue_statement", &node.rule_name))
}

pub fn parse_label<'input>(
    node: &NodeInternal<'input>,
) -> Result<Identifier<'input>, Box<dyn Error>> {
    if node.rule_name == "Label" {
        return Ok(parse_identifier(&node.children[0])?);
    }

    Err(gen_error("parse_label", &node.rule_name))
}
