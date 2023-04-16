use std::error::Error;

use super::{
    node::{
        FunctionParamSignature, FunctionParameter, FunctionSignature, Identifier, NodeStatement,
    },
    node_internal::NodeInternal,
    parse_expression::parse_expression,
    parse_identifier::parse_identifier,
    parse_statement::parse_block_statement,
    parse_type::parse_type,
    util::{flatten, gen_error},
};

pub fn parse_function_body<'input>(
    node: &NodeInternal<'input>,
) -> Result<NodeStatement<'input>, Box<dyn Error>> {
    if node.rule_name == "FunctionBody" {
        if node.children.len() == 1 {
            return parse_block_statement(&node.children[0]);
        } else {
            return Ok(NodeStatement::ExpressionStatement {
                expr: Box::new(parse_expression(&node.children[1])?),
            });
        }
    }

    Err(gen_error("parse_function_body", &node.rule_name))
}

pub fn parse_function_signature<'input>(
    node: &NodeInternal<'input>,
) -> Result<FunctionSignature<'input>, Box<dyn Error>> {
    if node.rule_name == "FunctionSignature" {
        if node.children.len() == 2 {
            return Ok(FunctionSignature {
                return_type: None,
                name: parse_identifier(&node.children[0])?,
                param: parse_formal_parameter_list(&node.children[1])?,
            });
        } else {
            return Ok(FunctionSignature {
                return_type: Some(parse_type(&node.children[0])?),
                name: parse_identifier(&node.children[1])?,
                param: parse_formal_parameter_list(&node.children[2])?,
            });
        }
    }

    Err(gen_error("parse_function_signature", &node.rule_name))
}

fn parse_formal_parameter_list<'input>(
    node: &NodeInternal<'input>,
) -> Result<FunctionParamSignature<'input>, Box<dyn Error>> {
    if node.rule_name == "FormalParameterList" {
        if node.children.len() == 2 {
            return Ok(FunctionParamSignature {
                normal_list: vec![],
                option_list: vec![],
                named_list: vec![],
            });
        } else if node.children.len() == 3 {
            let param = parse_optional_or_named_formal_parameter_list(&node.children[1])?;
            if param.1 {
                return Ok(FunctionParamSignature {
                    normal_list: vec![],
                    option_list: param.0,
                    named_list: vec![],
                });
            } else {
                return Ok(FunctionParamSignature {
                    normal_list: vec![],
                    option_list: vec![],
                    named_list: param.0,
                });
            }
        } else if node.children.len() == 4 {
            return Ok(FunctionParamSignature {
                normal_list: parse_normal_formal_parameter_list(&node.children[1])?,
                option_list: vec![],
                named_list: vec![],
            });
        } else {
            let param = parse_optional_or_named_formal_parameter_list(&node.children[3])?;
            if param.1 {
                return Ok(FunctionParamSignature {
                    normal_list: parse_normal_formal_parameter_list(&node.children[1])?,
                    option_list: param.0,
                    named_list: vec![],
                });
            } else {
                return Ok(FunctionParamSignature {
                    normal_list: parse_normal_formal_parameter_list(&node.children[1])?,
                    option_list: vec![],
                    named_list: param.0,
                });
            }
        }
    }

    Err(gen_error("parse_formal_parameter_list", &node.rule_name))
}

fn parse_optional_or_named_formal_parameter_list<'input>(
    node: &NodeInternal<'input>,
) -> Result<(Vec<FunctionParameter<'input>>, bool), Box<dyn Error>> {
    if node.rule_name == "OptionalOrNamedFormalParameterList" {
        if node.children[0].rule_name == "OptionalPositionalFormalParameterList" {
            return Ok((
                parse_optional_positional_formal_parameter_list(&node.children[0])?,
                true,
            ));
        } else {
            return Ok((parse_named_formal_parameter_list(&node.children[0])?, false));
        }
    }

    Err(gen_error(
        "parse_optional_or_named_formal_parameter_list",
        &node.rule_name,
    ))
}

fn parse_optional_positional_formal_parameter_list<'input>(
    node: &NodeInternal<'input>,
) -> Result<Vec<FunctionParameter<'input>>, Box<dyn Error>> {
    if node.rule_name == "OptionalPositionalFormalParameterList" {
        if node.children.len() == 2 {
            return Ok(vec![]);
        } else {
            return parse_optional_positional_formal_parameter_list_internal(&node.children[1]);
        }
    }

    Err(gen_error(
        "parse_optional_positional_formal_parameter_list",
        &node.rule_name,
    ))
}

fn parse_optional_positional_formal_parameter_list_internal<'input>(
    node: &NodeInternal<'input>,
) -> Result<Vec<FunctionParameter<'input>>, Box<dyn Error>> {
    if node.rule_name == "OptionalPositionalFormalParameterListInternal" {
        if node.children.len() == 1 {
            return Ok(vec![parse_default_formal_parameter(&node.children[0])?]);
        } else {
            return flatten(
                parse_optional_positional_formal_parameter_list_internal(&node.children[0]),
                parse_default_formal_parameter(&node.children[2])?,
            );
        }
    }

    Err(gen_error(
        "parse_optional_positional_formal_parameter_list_internal",
        &node.rule_name,
    ))
}

fn parse_named_formal_parameter_list<'input>(
    node: &NodeInternal<'input>,
) -> Result<Vec<FunctionParameter<'input>>, Box<dyn Error>> {
    if node.rule_name == "NamedFormalParameterList" {
        if node.children.len() == 2 {
            return Ok(vec![]);
        } else {
            return parse_named_formal_parameter_list_internal(&node.children[1]);
        }
    }

    Err(gen_error(
        "parse_named_formal_parameter_list",
        &node.rule_name,
    ))
}

fn parse_named_formal_parameter_list_internal<'input>(
    node: &NodeInternal<'input>,
) -> Result<Vec<FunctionParameter<'input>>, Box<dyn Error>> {
    if node.rule_name == "NamedFormalParameterListInternal" {
        if node.children.len() == 1 {
            return Ok(vec![parse_default_named_parameter(&node.children[0])?]);
        } else {
            return flatten(
                parse_named_formal_parameter_list_internal(&node.children[0]),
                parse_default_named_parameter(&node.children[2])?,
            );
        }
    }

    Err(gen_error(
        "parse_named_formal_parameter_list_internal",
        &node.rule_name,
    ))
}

fn parse_normal_formal_parameter<'input>(
    node: &NodeInternal<'input>,
) -> Result<Identifier<'input>, Box<dyn Error>> {
    if node.rule_name == "NormalFormalParameter" {
        if node.children[0].rule_name == "DeclaredIdentifier" {
            return Ok(parse_declared_identifier(&node.children[0])?);
        }
        return Ok(parse_identifier(&node.children[0])?);
    }

    Err(gen_error("parse_normal_formal_parameter", &node.rule_name))
}

fn parse_normal_formal_parameter_list<'input>(
    node: &NodeInternal<'input>,
) -> Result<Vec<FunctionParameter<'input>>, Box<dyn Error>> {
    if node.rule_name == "NormalFormalParameterList" {
        if node.children.len() == 1 {
            return Ok(vec![FunctionParameter {
                identifier: parse_normal_formal_parameter(&node.children[0])?,
                expr: None,
            }]);
        } else {
            return flatten(
                parse_normal_formal_parameter_list(&node.children[0]),
                FunctionParameter {
                    identifier: parse_normal_formal_parameter(&node.children[2])?,
                    expr: None,
                },
            );
        }
    }

    Err(gen_error(
        "parse_normal_formal_parameter_list",
        &node.rule_name,
    ))
}

fn parse_default_formal_parameter<'input>(
    node: &NodeInternal<'input>,
) -> Result<FunctionParameter<'input>, Box<dyn Error>> {
    if node.rule_name == "DefaultFormalParameter" {
        if node.children.len() == 1 {
            return Ok(FunctionParameter {
                identifier: parse_normal_formal_parameter(&node.children[0])?,
                expr: None,
            });
        } else {
            return Ok(FunctionParameter {
                identifier: parse_normal_formal_parameter(&node.children[0])?,
                expr: Some(Box::new(parse_expression(&node.children[2])?)),
            });
        }
    }

    Err(gen_error("parse_default_formal_parameter", &node.rule_name))
}

fn parse_default_named_parameter<'input>(
    node: &NodeInternal<'input>,
) -> Result<FunctionParameter<'input>, Box<dyn Error>> {
    if node.rule_name == "DefaultNamedParameter" {
        if node.children.len() == 1 {
            return Ok(FunctionParameter {
                identifier: if node.children[0].rule_name == "Identifier" {
                    parse_identifier(&node.children[0])?
                } else {
                    parse_declared_identifier(&node.children[0])?
                },
                expr: None,
            });
        }
        if node.children.len() == 2 {
            return Ok(FunctionParameter {
                identifier: if node.children[1].rule_name == "Identifier" {
                    parse_identifier(&node.children[1])?
                } else {
                    parse_declared_identifier(&node.children[1])?
                },
                expr: None,
            });
        }
        if node.children.len() == 3 {
            return Ok(FunctionParameter {
                identifier: if node.children[0].rule_name == "Identifier" {
                    parse_identifier(&node.children[0])?
                } else {
                    parse_declared_identifier(&node.children[0])?
                },
                expr: Some(Box::new(parse_expression(&node.children[2])?)),
            });
        }
        if node.children.len() == 4 {
            return Ok(FunctionParameter {
                identifier: if node.children[1].rule_name == "Identifier" {
                    parse_identifier(&node.children[1])?
                } else {
                    parse_declared_identifier(&node.children[1])?
                },
                expr: Some(Box::new(parse_expression(&node.children[3])?)),
            });
        }
    }

    Err(gen_error("parse_default_named_parameter", &node.rule_name))
}

pub fn parse_declared_identifier<'input>(
    node: &NodeInternal<'input>,
) -> Result<Identifier<'input>, Box<dyn Error>> {
    if node.rule_name == "DeclaredIdentifier" {
        return Ok(parse_identifier(&node.children.last().unwrap())?);
    }

    Err(gen_error("parse_declared_identifier", &node.rule_name))
}
