use std::fs;
use std::path::Path;
use std::rc::Rc;
use std::time::SystemTime;
use std::{cell::RefCell, collections::HashMap};

use anyhow::{bail, ensure, Result};

use crate::build_from_file;
use crate::bytecode::ByteCode;
use crate::executioncontext::{BlockContext, ExecutionContext, VariableScope};
use crate::parser::node::{
    CollectionElement, DartType, FunctionParamSignature, LibraryImport, NodeExpression,
    NodeStatement, Selector,
};
use crate::{bytecode::OpCode, pyobject::PyObject};

use self::runclass::run_class;
use self::runfunction::run_function;

pub mod runclass;
pub mod runfunction;
pub mod runroot;

struct DefaultScope {
    break_label: u32,
    continue_label: Option<u32>,
}

pub struct ByteCompiler<'ctx, 'value> {
    pub byte_operations: RefCell<Vec<OpCode>>,
    context_stack: Vec<Rc<RefCell<dyn ExecutionContext + 'ctx>>>,
    jump_label_table: RefCell<HashMap<u32, u8>>,
    jump_label_key_index: RefCell<u32>,
    default_scope_stack: Vec<DefaultScope>,
    break_label_table: HashMap<String, u32>,
    continue_label_table: HashMap<String, u32>,
    source: &'value str,
}

impl<'ctx, 'value> ByteCompiler<'ctx, 'value> {
    fn compile_import(&mut self, node: &LibraryImport, time_start_build: SystemTime) -> Result<()> {
        let uri = &node.uri;

        let identifier = match &node.identifier {
            Some(v) => Some(v.value),
            None => None,
        };

        // uri形式
        // import "elaphe/A/B.d.dart";
        // → from A.B import *
        // import "elaphe/A/B.d.dart" as C;
        // → import A.B as C
        // import "A.dart";
        // → from A import *
        // import "A.dart" as B;
        // → import A as B
        // 相対インポートは禁止
        // Schemeは禁止
        ensure!(!uri.contains(":"), "invalid import uri: {}", uri);

        // ignore relative representations
        // ex) ../../A/B/C -> A/B/C
        let mut path_splitted: Vec<&str> = uri.split("/").collect();
        let mut relative_count = 0;
        for i in 0..path_splitted.len() {
            if path_splitted[i].chars().all(|c| c == '.') {
                // ドットのみの場合は無視する
                relative_count += 1;
            } else {
                break;
            }
        }
        path_splitted.drain(0..relative_count);

        if path_splitted.is_empty() {
            return Ok(());
        }

        if path_splitted[0] == "elaphe" {
            // import Python modules
            let last = path_splitted.pop().unwrap();
            assert!(last.ends_with(".d.dart"));
            if last == "core.d.dart" {
                // ignore core.d.dart
                return Ok(());
            }

            // remove "elaphe"
            path_splitted.drain(0..1);

            // remove ".d.dart"
            let last = last.trim_end_matches(".d.dart");
            path_splitted.push(last);
        } else {
            // import Dart modules
            let output = Path::new(&uri).with_extension("pyc");
            if !output.exists() {
                build_from_file(output.to_str().unwrap(), &uri, time_start_build, false)?;
            } else {
                let metadata = fs::metadata(&output).unwrap();
                let modified = metadata.modified().unwrap();
                if time_start_build > modified {
                    // if the file is not modified, rebuild it
                    build_from_file(output.to_str().unwrap(), &uri, time_start_build, false)?;
                }
            }

            let last = path_splitted.pop().unwrap();
            assert!(last.ends_with(".dart"));

            // remove ".dart"
            let last = last.trim_end_matches(".dart");
            path_splitted.push(last);
        }

        match identifier {
            None => {
                // from A.B import *

                // 0を積む
                self.push_load_const(PyObject::Int(0, false));

                // ('*', )を積む
                self.push_load_const(PyObject::SmallTuple {
                    children: vec![PyObject::new_string("*".to_string(), false)],
                    add_ref: false,
                });

                // 名前でインポート
                let import_name = path_splitted.join(".");
                let import_name_p = (**self.context_stack.last().unwrap())
                    .borrow_mut()
                    .register_or_get_name(&import_name);
                self.push_op(OpCode::ImportName(import_name_p));

                self.push_op(OpCode::ImportStar);
            }
            Some(v) => {
                // 0を積む
                self.push_load_const(PyObject::Int(0, false));

                // Noneを積む
                self.push_load_const(PyObject::None(false));

                // 名前でインポート
                let import_name = path_splitted.join(".");
                let import_name_p = (**self.context_stack.last().unwrap())
                    .borrow_mut()
                    .register_or_get_name(&import_name);
                self.push_op(OpCode::ImportName(import_name_p));

                if path_splitted.len() == 1 {
                    // import A as B
                    let p = (**self.context_stack.last().unwrap())
                        .borrow_mut()
                        .declare_variable(&v.to_string());
                    self.push_op(OpCode::StoreName(p));
                } else {
                    // import A.B.C as D
                    let second_name = path_splitted[1].to_string();
                    let p = (**self.context_stack.last().unwrap())
                        .borrow_mut()
                        .register_or_get_name(&second_name);
                    self.push_op(OpCode::ImportFrom(p));
                    for i in 2..path_splitted.len() {
                        self.push_op(OpCode::RotTwo);
                        self.push_op(OpCode::PopTop);
                        let p = (**self.context_stack.last().unwrap())
                            .borrow_mut()
                            .register_or_get_name(&path_splitted[i].to_string());
                        self.push_op(OpCode::ImportFrom(p));
                    }
                    let p = (**self.context_stack.last().unwrap())
                        .borrow_mut()
                        .declare_variable(&v.to_string());
                    self.push_op(OpCode::StoreName(p));
                    self.push_op(OpCode::PopTop);
                }
            }
        }
        Ok(())
    }

    fn compile_expr(&mut self, node: &'value NodeExpression) -> Result<()> {
        match node {
            NodeExpression::Binary {
                left,
                operator,
                right,
            } => {
                if *operator == "??" {
                    self.compile_expr(left)?;
                    self.push_op(OpCode::DupTop);
                    self.push_load_const(PyObject::None(false));
                    self.push_op(OpCode::compare_op_from_str("==")?);
                    let label_end = self.gen_jump_label();
                    self.push_op(OpCode::PopJumpIfFalse(label_end));
                    self.push_op(OpCode::PopTop);
                    self.compile_expr(right)?;
                    self.set_jump_label_value(label_end);
                } else if *operator == "||" {
                    self.compile_expr(left)?;
                    self.push_op(OpCode::DupTop);
                    let label_end = self.gen_jump_label();
                    self.push_op(OpCode::PopJumpIfTrue(label_end));
                    self.push_op(OpCode::PopTop);
                    self.compile_expr(right)?;
                    self.set_jump_label_value(label_end);
                } else if *operator == "&&" {
                    self.compile_expr(left)?;
                    self.push_op(OpCode::DupTop);
                    let label_end = self.gen_jump_label();
                    self.push_op(OpCode::PopJumpIfFalse(label_end));
                    self.push_op(OpCode::PopTop);
                    self.compile_expr(right)?;
                    self.set_jump_label_value(label_end);
                } else {
                    self.compile_expr(left)?;
                    self.compile_expr(right)?;
                    match *operator {
                        "==" | "!=" | ">=" | ">" | "<=" | "<" => {
                            self.push_op(OpCode::compare_op_from_str(operator)?)
                        }
                        "<<" => self.push_op(OpCode::BinaryLShift),
                        ">>" => self.push_op(OpCode::BinaryRShift),
                        "&" => self.push_op(OpCode::BinaryAnd),
                        "^" => self.push_op(OpCode::BinaryXor),
                        "|" => self.push_op(OpCode::BinaryOr),
                        "+" => self.push_op(OpCode::BinaryAdd),
                        "-" => self.push_op(OpCode::BinarySubtract),
                        "*" => self.push_op(OpCode::BinaryMultiply),
                        "/" => self.push_op(OpCode::BinaryTrueDivide),
                        "%" => self.push_op(OpCode::BinaryModulo),
                        "~/" => self.push_op(OpCode::BinaryFloorDivide),
                        _ => bail!("unknown binary operator: {}", *operator),
                    }
                }
            }
            NodeExpression::Conditional {
                condition,
                true_expr,
                false_expr,
            } => {
                let label_conditional_end = self.gen_jump_label();
                let label_false_start = self.gen_jump_label();
                self.compile_expr(condition)?;
                self.push_op(OpCode::PopJumpIfFalse(label_false_start));
                self.compile_expr(true_expr)?;
                self.push_op(OpCode::JumpAbsolute(label_conditional_end));
                self.set_jump_label_value(label_false_start);
                self.compile_expr(false_expr)?;
                self.set_jump_label_value(label_conditional_end);
            }
            NodeExpression::Unary { operator, expr } => {
                self.compile_expr(expr)?;
                match *operator {
                    "-" => self.push_op(OpCode::UnaryNegative),
                    "!" => self.push_op(OpCode::UnaryNot),
                    "~" => self.push_op(OpCode::UnaryInvert),
                    _ => bail!("unknown unary operator: {}", *operator),
                }
            }
            NodeExpression::Update {
                operator,
                is_prefix,
                child,
            } => {
                if let NodeExpression::Identifier { identifier } = &**child {
                    let value = identifier.value.to_string();
                    if *is_prefix {
                        // 前置
                        self.push_load_var(&value);
                        self.push_load_const(PyObject::Int(1, false));
                        match *operator {
                            "++" => self.push_op(OpCode::InplaceAdd),
                            "--" => self.push_op(OpCode::InplaceSubtract),
                            _ => (),
                        }
                        self.push_op(OpCode::DupTop);
                        self.push_store_var(&value);
                    } else {
                        // 後置
                        self.push_load_var(&value);
                        self.push_op(OpCode::DupTop);
                        self.push_load_const(PyObject::Int(1, false));
                        match *operator {
                            "++" => self.push_op(OpCode::InplaceAdd),
                            "--" => self.push_op(OpCode::InplaceSubtract),
                            _ => (),
                        }
                        self.push_store_var(&value);
                    }
                } else {
                    bail!("Invalid AST. Increment target must be an identifier.");
                }
            }
            NodeExpression::TypeTest { child, type_test } => {
                // isinstance(child, type_test.)
                self.push_load_var(&"isinstance".to_string());
                self.compile_expr(child)?;
                if let DartType::Named {
                    type_name,
                    type_arguments: _,
                    is_nullable: _,
                } = &type_test.dart_type
                {
                    let name = type_name.identifier.value;
                    let p = (**self.context_stack.last().unwrap())
                        .borrow_mut()
                        .register_or_get_name(&name.to_string());
                    self.push_op(OpCode::LoadName(p));
                } else {
                    bail!("Invalid Test Expression");
                }
                self.push_op(OpCode::CallFunction(2));
                if !type_test.check_matching {
                    self.push_op(OpCode::UnaryNot);
                }
            }
            NodeExpression::TypeCast {
                child,
                type_cast: _,
            } => {
                // 実行時には型がないので無視
                self.compile_expr(child)?;
            }
            NodeExpression::Assignment {
                operator,
                left,
                right,
            } => {
                match *operator {
                    "=" => {
                        self.compile_expr(right)?;
                        // DartではAssignment Expressionが代入先の最終的な値を残す
                        self.push_op(OpCode::DupTop);

                        match &**left {
                            NodeExpression::Identifier { identifier } => {
                                let value = identifier.value.to_string();
                                self.push_store_var(&value);
                            }
                            NodeExpression::Selector { child, selector } => {
                                self.compile_expr(child)?;

                                match selector {
                                    Selector::Args { args: _ } => {
                                        bail!("Invalid lhs value. Function call is not allowed.")
                                    }
                                    Selector::Method {
                                        identifier: _,
                                        arguments: _,
                                    } => bail!("Invalid lhs value. Method call is not allowed."),
                                    Selector::Attr { identifier } => {
                                        let name = identifier.value;
                                        let p = (**self.context_stack.last().unwrap())
                                            .borrow_mut()
                                            .register_or_get_name(&name.to_string());
                                        self.push_op(OpCode::StoreAttr(p));
                                    }
                                    Selector::Index { expr } => {
                                        self.compile_expr(expr)?;
                                        self.push_op(OpCode::StoreSubScr);
                                    }
                                }
                            }
                            _ => bail!("Invalid lhs value."),
                        }
                    }
                    "*=" | "/=" | "~/=" | "%=" | "+=" | "-=" | "<<=" | ">>=" | "&=" | "^="
                    | "|=" => match &**left {
                        NodeExpression::Identifier { identifier } => {
                            let value = identifier.value.to_string();
                            self.push_load_var(&value);

                            self.compile_expr(right)?;
                            match *operator {
                                "*=" => self.push_op(OpCode::InplaceMultiply),
                                "/=" => self.push_op(OpCode::InplaceTrueDivide),
                                "~/=" => self.push_op(OpCode::InplaceFloorDivide),
                                "%=" => self.push_op(OpCode::InplaceModulo),
                                "+=" => self.push_op(OpCode::InplaceAdd),
                                "-=" => self.push_op(OpCode::InplaceSubtract),
                                "<<=" => self.push_op(OpCode::InplaceLShift),
                                ">>=" => self.push_op(OpCode::InplaceRShift),
                                "&=" => self.push_op(OpCode::InplaceAnd),
                                "^=" => self.push_op(OpCode::InplaceXor),
                                "|=" => self.push_op(OpCode::InplaceOr),
                                _ => (),
                            }
                            self.push_op(OpCode::DupTop);

                            self.push_store_var(&value);
                        }
                        NodeExpression::Selector { child, selector } => {
                            self.compile_expr(child)?;
                            match selector {
                                Selector::Args { args: _ } => {
                                    bail!("Invalid lhs value. Function call is not allowed.")
                                }
                                Selector::Method {
                                    identifier: _,
                                    arguments: _,
                                } => bail!("Invalid lhs value. Method call is not allowed."),
                                Selector::Attr { identifier } => {
                                    let name = identifier.value;
                                    let p = (**self.context_stack.last().unwrap())
                                        .borrow_mut()
                                        .register_or_get_name(&name.to_string());

                                    self.push_op(OpCode::DupTop);
                                    self.push_op(OpCode::LoadAttr(p));

                                    self.compile_expr(right)?;
                                    match *operator {
                                        "*=" => self.push_op(OpCode::InplaceMultiply),
                                        "/=" => self.push_op(OpCode::InplaceTrueDivide),
                                        "~/=" => self.push_op(OpCode::InplaceFloorDivide),
                                        "%=" => self.push_op(OpCode::InplaceModulo),
                                        "+=" => self.push_op(OpCode::InplaceAdd),
                                        "-=" => self.push_op(OpCode::InplaceSubtract),
                                        "<<=" => self.push_op(OpCode::InplaceLShift),
                                        ">>=" => self.push_op(OpCode::InplaceRShift),
                                        "&=" => self.push_op(OpCode::InplaceAnd),
                                        "^=" => self.push_op(OpCode::InplaceXor),
                                        "|=" => self.push_op(OpCode::InplaceOr),
                                        _ => (),
                                    }
                                    self.push_op(OpCode::DupTop);

                                    self.push_op(OpCode::RotThree);
                                    self.push_op(OpCode::StoreAttr(p));
                                }
                                Selector::Index { expr } => {
                                    self.compile_expr(expr)?;
                                    self.push_op(OpCode::DupTopTwo);
                                    self.push_op(OpCode::BinarySubScr);

                                    self.compile_expr(right)?;
                                    match *operator {
                                        "*=" => self.push_op(OpCode::InplaceMultiply),
                                        "/=" => self.push_op(OpCode::InplaceTrueDivide),
                                        "~/=" => self.push_op(OpCode::InplaceFloorDivide),
                                        "%=" => self.push_op(OpCode::InplaceModulo),
                                        "+=" => self.push_op(OpCode::InplaceAdd),
                                        "-=" => self.push_op(OpCode::InplaceSubtract),
                                        "<<=" => self.push_op(OpCode::InplaceLShift),
                                        ">>=" => self.push_op(OpCode::InplaceRShift),
                                        "&=" => self.push_op(OpCode::InplaceAnd),
                                        "^=" => self.push_op(OpCode::InplaceXor),
                                        "|=" => self.push_op(OpCode::InplaceOr),
                                        _ => (),
                                    }
                                    self.push_op(OpCode::DupTop);
                                    self.push_op(OpCode::RotFour);

                                    self.push_op(OpCode::RotFour);
                                    self.push_op(OpCode::StoreSubScr);
                                }
                            }
                        }
                        _ => bail!("Invalid lhs value."),
                    },
                    "??=" => match &**left {
                        NodeExpression::Identifier { identifier } => {
                            let value = identifier.value.to_string();

                            self.push_load_var(&value);
                            self.push_op(OpCode::DupTop);
                            self.push_load_const(PyObject::None(false));
                            self.push_op(OpCode::compare_op_from_str("==")?);
                            let label_end = self.gen_jump_label();
                            self.push_op(OpCode::PopJumpIfFalse(label_end));

                            self.push_op(OpCode::PopTop);
                            self.compile_expr(right)?;
                            self.push_op(OpCode::DupTop);
                            self.push_store_var(&value);
                            self.set_jump_label_value(label_end);
                        }
                        NodeExpression::Selector { child, selector } => {
                            self.compile_expr(child)?;
                            match selector {
                                Selector::Args { args: _ } => {
                                    bail!("Invalid lhs value. Function call is not allowed.")
                                }
                                Selector::Method {
                                    identifier: _,
                                    arguments: _,
                                } => bail!("Invalid lhs value. Method call is not allowed."),
                                Selector::Attr { identifier } => {
                                    let name = identifier.value;
                                    let p = (**self.context_stack.last().unwrap())
                                        .borrow_mut()
                                        .register_or_get_name(&name.to_string());

                                    self.push_op(OpCode::DupTop);
                                    self.push_op(OpCode::LoadAttr(p));

                                    self.push_op(OpCode::DupTop);
                                    self.push_load_const(PyObject::None(false));
                                    self.push_op(OpCode::compare_op_from_str("==")?);
                                    let label_false = self.gen_jump_label();
                                    self.push_op(OpCode::PopJumpIfFalse(label_false));

                                    self.push_op(OpCode::PopTop);
                                    self.compile_expr(right)?;
                                    self.push_op(OpCode::DupTop);
                                    self.push_op(OpCode::RotThree);
                                    self.push_op(OpCode::RotThree);
                                    self.push_op(OpCode::StoreAttr(p));
                                    let label_end = self.gen_jump_label();
                                    self.push_op(OpCode::JumpAbsolute(label_end));

                                    self.set_jump_label_value(label_false);
                                    self.push_op(OpCode::RotTwo);
                                    self.push_op(OpCode::PopTop);

                                    self.set_jump_label_value(label_end);
                                }
                                Selector::Index { expr } => {
                                    self.compile_expr(expr)?;
                                    self.push_op(OpCode::DupTopTwo);
                                    self.push_op(OpCode::BinarySubScr);

                                    self.push_op(OpCode::DupTop);
                                    self.push_load_const(PyObject::None(false));
                                    self.push_op(OpCode::compare_op_from_str("==")?);
                                    let label_false = self.gen_jump_label();
                                    self.push_op(OpCode::PopJumpIfFalse(label_false));

                                    self.push_op(OpCode::PopTop);
                                    self.compile_expr(right)?;
                                    self.push_op(OpCode::DupTop);
                                    self.push_op(OpCode::RotFour);
                                    self.push_op(OpCode::RotFour);
                                    self.push_op(OpCode::StoreSubScr);
                                    let label_end = self.gen_jump_label();
                                    self.push_op(OpCode::JumpAbsolute(label_end));

                                    self.set_jump_label_value(label_false);
                                    self.push_op(OpCode::RotThree);
                                    self.push_op(OpCode::PopTop);
                                    self.push_op(OpCode::PopTop);

                                    self.set_jump_label_value(label_end);
                                }
                            }
                        }
                        _ => bail!("Invalid lhs value."),
                    },
                    _ => bail!("Unknown assignment operator: {}", operator),
                }
            }
            NodeExpression::NumericLiteral { value } => {
                self.push_load_const(PyObject::new_numeric(value, false)?);
            }
            NodeExpression::StringLiteral { str_list } => {
                for (i, single_str) in str_list.iter().enumerate() {
                    for (i, str) in single_str.string_list.iter().enumerate() {
                        // string + str(expr) + string + str(expr) + ...
                        if i > 0 {
                            // str(expr)
                            self.push_load_var(&"str".to_string());
                            self.compile_expr(&single_str.interpolation_list[i - 1])?;
                            self.push_op(OpCode::CallFunction(1));

                            self.push_op(OpCode::BinaryAdd);
                        }
                        self.push_load_const(PyObject::new_string(replace_escape(&str)?, false));
                        if i > 0 {
                            self.push_op(OpCode::BinaryAdd);
                        }
                    }
                    if i > 0 {
                        self.push_op(OpCode::BinaryAdd);
                    }
                }
                // let value = str_list
                //     .iter()
                //     .map(|v| {
                //         let len = v.len();
                //         &v[1..len - 1]
                //     })
                //     .collect::<Vec<&'value str>>()
                //     .join("");

                // self.push_load_const(PyObject::new_string(value.to_string(), false));
            }
            NodeExpression::BooleanLiteral { value } => {
                self.push_load_const(PyObject::new_boolean(value, false)?);
            }
            NodeExpression::NullLiteral => {
                self.push_load_const(PyObject::None(false));
            }
            NodeExpression::ListLiteral { element_list } => {
                let size = element_list.len() as u8;
                for elem in element_list {
                    match elem {
                        CollectionElement::ExpressionElement { expr } => {
                            self.compile_expr(expr)?;
                        }
                        CollectionElement::MapElement {
                            key_expr: _,
                            value_expr: _,
                        } => {
                            bail!("Invalid List Literal. Map is not allowed.");
                        }
                    }
                }
                self.push_op(OpCode::BuildList(size));
            }
            NodeExpression::SetOrMapLiteral { element_list } => {
                let first_elem = element_list.first();
                let is_map = if let Some(elem) = first_elem {
                    match elem {
                        CollectionElement::ExpressionElement { expr: _ } => false,
                        CollectionElement::MapElement {
                            key_expr: _,
                            value_expr: _,
                        } => true,
                    }
                } else {
                    true
                };

                if is_map {
                    let size = element_list.len() as u8;
                    for elem in element_list {
                        match elem {
                            CollectionElement::ExpressionElement { expr: _ } => {
                                bail!("Invalid Map Literal. Expression is not allowed.");
                            }
                            CollectionElement::MapElement {
                                key_expr,
                                value_expr,
                            } => {
                                self.compile_expr(key_expr)?;
                                self.compile_expr(value_expr)?;
                            }
                        }
                    }
                    self.push_op(OpCode::BuildMap(size));
                } else {
                    let size = element_list.len() as u8;
                    for elem in element_list {
                        match elem {
                            CollectionElement::ExpressionElement { expr } => {
                                self.compile_expr(expr)?;
                            }
                            CollectionElement::MapElement {
                                key_expr: _,
                                value_expr: _,
                            } => {
                                bail!("Invalid Set Literal. Map is not allowed.");
                            }
                        }
                    }
                    self.push_op(OpCode::BuildSet(size));
                }
            }
            NodeExpression::Identifier { identifier } => {
                let value = identifier.value.to_string();
                self.push_load_var(&value);
            }
            NodeExpression::Selector { child, selector } => {
                // 右辺値として処理される場合
                self.compile_expr(child)?;

                match selector {
                    Selector::Args { args } => {
                        let mut name_list: Vec<&str> = vec![];
                        for param in args {
                            self.compile_expr(&param.expr)?;
                            if let Some(v) = &param.identifier {
                                name_list.push(v.value);
                            }
                        }
                        if !name_list.is_empty() {
                            self.push_load_const(PyObject::SmallTuple {
                                children: name_list
                                    .iter()
                                    .map(|v| PyObject::new_string(v.to_string(), false))
                                    .collect(),
                                add_ref: false,
                            });
                            self.push_op(OpCode::CallFunctionKw(args.len() as u8));
                        } else {
                            self.push_op(OpCode::CallFunction(args.len() as u8));
                        }
                    }
                    Selector::Attr { identifier } => {
                        let name = identifier.value;
                        let p = (**self.context_stack.last().unwrap())
                            .borrow_mut()
                            .register_or_get_name(&name.to_string());
                        self.push_op(OpCode::LoadAttr(p));
                    }
                    Selector::Index { expr } => {
                        self.compile_expr(expr)?;

                        self.push_op(OpCode::BinarySubScr);
                    }
                    Selector::Method {
                        identifier,
                        arguments,
                    } => {
                        let name = identifier.value;
                        let p = (**self.context_stack.last().unwrap())
                            .borrow_mut()
                            .register_or_get_name(&name.to_string());
                        self.push_op(OpCode::LoadMethod(p));

                        let mut name_list: Vec<&str> = vec![];
                        for param in arguments {
                            self.compile_expr(&param.expr)?;
                            if let Some(v) = &param.identifier {
                                name_list.push(&v.value);
                            }
                        }
                        if !name_list.is_empty() {
                            self.push_load_const(PyObject::SmallTuple {
                                children: name_list
                                    .iter()
                                    .map(|v| PyObject::new_string(v.to_string(), false))
                                    .collect(),
                                add_ref: false,
                            });
                            self.push_op(OpCode::CallFunctionKw(arguments.len() as u8));
                        } else {
                            self.push_op(OpCode::CallMethod(arguments.len() as u8))
                        }
                    }
                }
            }
            NodeExpression::Slice { start, end, step } => {
                match start {
                    Some(v) => {
                        self.compile_expr(v)?;
                    }
                    None => {
                        self.push_load_const(PyObject::None(false));
                    }
                }

                match end {
                    Some(v) => {
                        self.compile_expr(v)?;
                    }
                    None => {
                        self.push_load_const(PyObject::None(false));
                    }
                }

                match step {
                    Some(v) => {
                        self.compile_expr(v)?;
                        self.push_op(OpCode::BuildSlice(3));
                    }
                    None => {
                        self.push_op(OpCode::BuildSlice(2));
                    }
                }
            }
            NodeExpression::Throw { expr } => {
                self.compile_expr(expr)?;
                self.push_op(OpCode::RaiseVarargs(1));
            }
            NodeExpression::This => {
                let p = self
                    .context_stack
                    .last()
                    .unwrap()
                    .borrow()
                    .get_local_variable(&"self".to_string());
                self.push_op(OpCode::LoadFast(p));
            }
        }
        Ok(())
    }

    fn compile_stmt(&mut self, node: &'value NodeStatement, label: Option<&String>) -> Result<()> {
        match node {
            NodeStatement::Labeled { label, stmt } => {
                let label_str = label.value.to_string();
                let label_id = self.gen_jump_label();

                // break用のラベルはこの時点で用意する
                self.break_label_table.insert(label_str.clone(), label_id);

                self.compile_stmt(stmt, Some(&label_str))?;

                self.set_jump_label_value(label_id);
                self.break_label_table.remove(&label_str);
            }
            NodeStatement::Break { label } => match label {
                Some(identifier) => {
                    let label_str = identifier.value;
                    match self.break_label_table.get(label_str) {
                        Some(v) => {
                            self.push_op(OpCode::JumpAbsolute(*v));
                        }
                        None => bail!("label {} is not existing in this scope.", label_str),
                    }
                }
                None => match self.default_scope_stack.last() {
                    Some(v) => {
                        self.push_op(OpCode::JumpAbsolute(v.break_label));
                    }
                    None => bail!("break statement is not available here"),
                },
            },
            NodeStatement::Continue { label } => match label {
                Some(identifier) => {
                    let label_str = identifier.value;
                    match self.continue_label_table.get(label_str) {
                        Some(v) => {
                            self.push_op(OpCode::JumpAbsolute(*v));
                        }
                        None => bail!("label {} is not existing in this scope.", label_str),
                    }
                }
                None => match self.default_scope_stack.last() {
                    Some(v) => match v.continue_label {
                        Some(continue_label) => self.push_op(OpCode::JumpAbsolute(continue_label)),
                        None => bail!("continue statement is not available here"),
                    },
                    None => bail!("continue statement is not available here"),
                },
            },
            NodeStatement::Return { value } => {
                match value {
                    Some(v) => {
                        self.compile_expr(v)?;
                    }
                    None => {
                        self.push_load_const(PyObject::None(false));
                    }
                }
                self.push_op(OpCode::ReturnValue);
            }
            NodeStatement::Empty => {}
            NodeStatement::Expression { expr } => {
                self.compile_expr(expr)?;
                self.push_op(OpCode::PopTop);
            }
            NodeStatement::Block { statements } => {
                self.context_stack.push(Rc::new(RefCell::new(BlockContext {
                    outer: self.context_stack.last().unwrap().clone(),
                    variables: vec![],
                })));
                for child in statements {
                    self.compile_stmt(child, None)?;
                }
                self.context_stack.pop();
            }
            NodeStatement::Rethrow => {
                self.push_op(OpCode::Reraise);
            }
            NodeStatement::VariableDeclarationList { decl_list } => {
                for declaration in decl_list {
                    match &declaration.expr {
                        Some(e) => {
                            self.compile_expr(e)?;
                            let value = declaration.identifier.value.to_string();
                            let position = (**self.context_stack.last().unwrap())
                                .borrow_mut()
                                .declare_variable(&value);
                            if self.context_stack.last().unwrap().borrow().is_global() {
                                // トップレベル変数の場合
                                self.push_op(OpCode::StoreName(position));
                            } else {
                                // ローカル変数の場合
                                let local_position = self
                                    .context_stack
                                    .last()
                                    .unwrap()
                                    .borrow()
                                    .get_local_variable(&value);
                                self.push_op(OpCode::StoreFast(local_position));
                            }
                        }
                        None => {
                            let value = declaration.identifier.value.to_string();
                            (**self.context_stack.last().unwrap())
                                .borrow_mut()
                                .declare_variable(&value);
                        }
                    }
                }
            }
            NodeStatement::FunctionDeclaration { signature, body } => {
                self.compile_declare_function(
                    &signature.name.value.to_string(),
                    &signature.param,
                    body,
                    None,
                    None,
                    |_| Ok(()),
                )?;
            }
            NodeStatement::ClassDeclaration {
                identifier,
                member_list,
            } => {
                self.push_op(OpCode::LoadBuildClass);

                let name = identifier.value.to_string();
                self.push_load_const(run_class(
                    &"main.py".to_string(),
                    &name,
                    member_list,
                    self,
                    self.source,
                )?);

                self.push_load_const(PyObject::new_string(name.clone(), false));

                self.push_op(OpCode::MakeFunction(0));

                self.push_load_const(PyObject::new_string(name.clone(), false));

                self.push_op(OpCode::CallFunction(2));

                let p = (**self.context_stack.last().unwrap())
                    .borrow_mut()
                    .declare_variable(&name);
                self.push_op(OpCode::StoreName(p));
            }
            NodeStatement::If {
                condition,
                if_true_stmt,
                if_false_stmt,
            } => {
                match if_false_stmt {
                    Some(if_false_stmt) => {
                        // if expr stmt else stmt
                        self.compile_expr(condition)?;
                        let label_false_starts = self.gen_jump_label();
                        self.push_op(OpCode::PopJumpIfFalse(label_false_starts));

                        self.context_stack.push(Rc::new(RefCell::new(BlockContext {
                            outer: self.context_stack.last().unwrap().clone(),
                            variables: vec![],
                        })));
                        self.compile_stmt(if_true_stmt, None)?;
                        self.context_stack.pop();

                        let label_if_ends = self.gen_jump_label();
                        self.push_op(OpCode::JumpAbsolute(label_if_ends));

                        self.set_jump_label_value(label_false_starts);

                        self.context_stack.push(Rc::new(RefCell::new(BlockContext {
                            outer: self.context_stack.last().unwrap().clone(),
                            variables: vec![],
                        })));
                        self.compile_stmt(if_false_stmt, None)?;
                        self.context_stack.pop();

                        self.set_jump_label_value(label_if_ends);
                    }
                    None => {
                        // if expr stmt
                        self.compile_expr(condition)?;
                        let label_if_ends = self.gen_jump_label();
                        self.push_op(OpCode::PopJumpIfFalse(label_if_ends));
                        self.compile_stmt(if_true_stmt, None)?;

                        self.set_jump_label_value(label_if_ends);
                    }
                }
            }
            NodeStatement::TryFinally {
                block_try,
                block_finally,
            } => {
                let label_finally = self.gen_jump_label();
                let label_end = self.gen_jump_label();

                self.push_op(OpCode::SetupFinally(label_finally));
                let label_finally_zero = self.byte_operations.borrow().len() as u8;

                // 通常フロー
                self.compile_stmt(block_try, None)?;
                self.push_op(OpCode::PopBlock);
                self.compile_stmt(block_finally, None)?;
                self.push_op(OpCode::JumpAbsolute(label_end));

                // 例外が起きたときのフロー
                self.set_jump_label_value_offset(label_finally, label_finally_zero);
                self.compile_stmt(block_finally, None)?;
                self.push_op(OpCode::Reraise);

                self.set_jump_label_value(label_end);
            }
            NodeStatement::TryOn {
                block_try,
                on_part_list,
            } => {
                let label_finally = self.gen_jump_label();
                let label_end = self.gen_jump_label();

                self.push_op(OpCode::SetupFinally(label_finally));
                let label_finally_zero = self.byte_operations.borrow().len() as u8;

                // 通常のフロー
                self.compile_stmt(block_try, None)?;
                self.push_op(OpCode::PopBlock);
                self.push_op(OpCode::JumpAbsolute(label_end));

                // 例外時のフロー
                self.set_jump_label_value_offset(label_finally, label_finally_zero);
                for on_part in on_part_list {
                    let label_next = self.gen_jump_label();

                    self.context_stack.push(Rc::new(RefCell::new(BlockContext {
                        outer: self.context_stack.last().unwrap().clone(),
                        variables: vec![],
                    })));

                    // catchする型の指定がある場合はloadして検証する
                    match &on_part.exc_type {
                        Some(v) => {
                            if let DartType::Named {
                                type_name,
                                type_arguments: _,
                                is_nullable: _,
                            } = v
                            {
                                let name = type_name.identifier.value.to_string();
                                let p = (**self.context_stack.last().unwrap())
                                    .borrow_mut()
                                    .register_or_get_name(&name);
                                self.push_op(OpCode::LoadName(p));
                                self.push_op(OpCode::JumpIfNotExcMatch(label_next));
                            }
                        }
                        None => (),
                    }

                    // スタックの状態:
                    // [trace_back, value, exception] -> TOP
                    match &on_part.catch_part {
                        Some(catch_part) => {
                            // on E catch() { }
                            // self.push_op(OpCode::PopTop);
                            // 実行してみると最初にpopしたものがexception、
                            // 2番目がtraceback objectになっている
                            // なぜ？

                            let name = catch_part.id_error.value.to_string();
                            (**self.context_stack.last().unwrap())
                                .borrow_mut()
                                .declare_variable(&name);
                            self.push_store_var(&name);

                            match &catch_part.id_trace {
                                Some(id_trace) => {
                                    let name = id_trace.value.to_string();
                                    (**self.context_stack.last().unwrap())
                                        .borrow_mut()
                                        .declare_variable(&name);
                                    self.push_store_var(&name);
                                }
                                None => {
                                    self.push_op(OpCode::PopTop);
                                }
                            }

                            self.push_op(OpCode::PopTop);
                        }
                        None => {
                            // on E { }
                            self.push_op(OpCode::PopTop);
                            self.push_op(OpCode::PopTop);
                            self.push_op(OpCode::PopTop);
                        }
                    }

                    self.compile_stmt(&on_part.block, None)?;
                    self.push_op(OpCode::PopExcept);
                    self.push_op(OpCode::JumpAbsolute(label_end));

                    self.set_jump_label_value(label_next);

                    self.context_stack.pop();
                }

                self.push_op(OpCode::Reraise);

                self.set_jump_label_value(label_end);
            }
            NodeStatement::For {
                init,
                condition,
                update,
                stmt,
            } => {
                let label_for_end = self.gen_jump_label();
                let label_loop_start = self.gen_jump_label();
                self.default_scope_stack.push(DefaultScope {
                    break_label: label_for_end,
                    continue_label: Some(label_loop_start),
                });
                if let Some(stmt_label) = label {
                    self.continue_label_table
                        .insert(stmt_label.to_string(), label_loop_start);
                }
                if let Some(node) = init {
                    self.compile_stmt(node, None)?;
                }
                self.set_jump_label_value(label_loop_start);
                if let Some(node) = condition {
                    self.compile_expr(node)?;
                    self.push_op(OpCode::PopJumpIfFalse(label_for_end));
                }

                self.context_stack.push(Rc::new(RefCell::new(BlockContext {
                    outer: self.context_stack.last().unwrap().clone(),
                    variables: vec![],
                })));
                self.compile_stmt(stmt, None)?;
                self.context_stack.pop();

                if let Some(node_list) = update {
                    for node in node_list {
                        self.compile_expr(node)?;
                        self.push_op(OpCode::PopTop);
                    }
                }
                self.push_op(OpCode::JumpAbsolute(label_loop_start));
                self.set_jump_label_value(label_for_end);
                self.default_scope_stack.pop();
                if let Some(stmt_label) = label {
                    self.continue_label_table.remove(stmt_label);
                }
            }
            NodeStatement::ForIn {
                variable,
                is_variable_declared,
                iterable,
                stmt,
            } => {
                if *is_variable_declared {
                    (**self.context_stack.last().unwrap())
                        .borrow_mut()
                        .declare_variable(&variable.value.to_string());
                }
                self.compile_expr(iterable)?;
                self.push_op(OpCode::GetIter);

                let label_for_end = self.gen_jump_label();
                let label_loop_start = self.gen_jump_label();

                self.set_jump_label_value(label_loop_start);
                self.push_op(OpCode::ForIter(label_for_end));
                let label_for_zero = self.byte_operations.borrow().len() as u8;

                let var_name = variable.value.to_string();
                self.push_store_var(&var_name);

                self.compile_stmt(stmt, None)?;

                self.push_op(OpCode::JumpAbsolute(label_loop_start));

                self.set_jump_label_value_offset(label_for_end, label_for_zero);
            }
            NodeStatement::While { condition, stmt } => {
                let label_while_end = self.gen_jump_label();
                let label_loop_start = self.gen_jump_label();
                self.default_scope_stack.push(DefaultScope {
                    break_label: label_while_end,
                    continue_label: Some(label_loop_start),
                });
                if let Some(stmt_label) = label {
                    self.continue_label_table
                        .insert(stmt_label.to_string(), label_loop_start);
                }
                self.set_jump_label_value(label_loop_start);
                self.compile_expr(condition)?;
                self.push_op(OpCode::PopJumpIfFalse(label_while_end));

                self.context_stack.push(Rc::new(RefCell::new(BlockContext {
                    outer: self.context_stack.last().unwrap().clone(),
                    variables: vec![],
                })));
                self.compile_stmt(stmt, None)?;
                self.context_stack.pop();

                self.push_op(OpCode::JumpAbsolute(label_loop_start));

                self.set_jump_label_value(label_while_end);

                self.default_scope_stack.pop();
                if let Some(stmt_label) = label {
                    self.continue_label_table.remove(stmt_label);
                }
            }
            NodeStatement::Do { condition, stmt } => {
                let label_do_start = self.gen_jump_label();
                let label_do_end = self.gen_jump_label();
                self.default_scope_stack.push(DefaultScope {
                    break_label: label_do_end,
                    continue_label: Some(label_do_start),
                });
                if let Some(stmt_label) = label {
                    self.continue_label_table
                        .insert(stmt_label.to_string(), label_do_start);
                }
                self.set_jump_label_value(label_do_start);

                self.context_stack.push(Rc::new(RefCell::new(BlockContext {
                    outer: self.context_stack.last().unwrap().clone(),
                    variables: vec![],
                })));
                self.compile_stmt(stmt, None)?;
                self.context_stack.pop();

                self.compile_expr(condition)?;
                self.push_op(OpCode::PopJumpIfTrue(label_do_start));
                self.set_jump_label_value(label_do_end);

                self.default_scope_stack.pop();
                if let Some(stmt_label) = label {
                    self.continue_label_table.remove(stmt_label);
                }
            }
            NodeStatement::Switch {
                expr,
                case_list,
                default_case,
            } => {
                self.compile_expr(expr)?;
                let label_switch_end = self.gen_jump_label();
                self.default_scope_stack.push(DefaultScope {
                    break_label: label_switch_end,
                    continue_label: match self.default_scope_stack.last() {
                        Some(v) => v.continue_label,
                        None => None,
                    },
                });
                let case_labels: Vec<u32> =
                    case_list.iter().map(|_| self.gen_jump_label()).collect();
                for case_index in 0..case_list.len() {
                    let case = &case_list[case_index];
                    let case_label = case_labels[case_index];
                    self.push_op(OpCode::DupTop);
                    self.compile_expr(&case.expr)?;
                    self.push_op(OpCode::compare_op_from_str("==")?);
                    self.push_op(OpCode::PopJumpIfTrue(case_label));
                }
                let label_default_start = self.gen_jump_label();
                if let Some(_) = default_case {
                    self.push_op(OpCode::JumpAbsolute(label_default_start));
                }
                for case_index in 0..case_list.len() {
                    let case = &case_list[case_index];
                    let case_label = case_labels[case_index];
                    self.set_jump_label_value(case_label);
                    for stmt in &case.stmt_list {
                        self.compile_stmt(stmt, None)?;
                    }
                }
                if let Some(default_case) = default_case {
                    self.set_jump_label_value(label_default_start);
                    for stmt in &default_case.stmt_list {
                        self.compile_stmt(stmt, None)?;
                    }
                }
                self.set_jump_label_value(label_switch_end);
                self.default_scope_stack.pop();
            }
        }
        Ok(())
    }

    fn compile_declare_function<F: FnOnce(&mut ByteCompiler<'ctx, 'value>) -> Result<()>>(
        &mut self,
        name: &String,
        param: &'value FunctionParamSignature,
        body: &'value Box<NodeStatement>,
        function_name_prefix: Option<String>,
        implicit_arg: Option<&String>,
        preface: F,
    ) -> Result<()> {
        let mut argument_list: Vec<String> = vec![];
        if let Some(name) = implicit_arg {
            argument_list.push(name.clone());
        }
        for p in &param.normal_list {
            let name = p.identifier.value.to_string();
            argument_list.push(name);
        }
        for p in &param.option_list {
            let name = p.identifier.value.to_string();
            argument_list.push(name);
        }
        for p in &param.named_list {
            let name = p.identifier.value.to_string();
            argument_list.push(name);
        }

        let mut num_normal_args = param.normal_list.len() as u32 + param.option_list.len() as u32;
        if implicit_arg.is_some() {
            num_normal_args += 1;
        }
        let num_kw_only_args = param.named_list.len() as u32;
        let py_code = run_function(
            &"main.py".to_string(),
            &name,
            argument_list,
            num_normal_args,
            0,
            num_kw_only_args,
            self,
            body,
            self.source,
            preface,
        )?;

        // 通常引数のデフォルト値の設定
        let has_default = !param.option_list.is_empty();
        if has_default {
            let size = param.option_list.len() as u8;
            for v in &param.option_list {
                match &v.expr {
                    Some(expr) => {
                        self.compile_expr(expr)?;
                    }
                    None => {
                        self.push_load_const(PyObject::None(false));
                    }
                }
            }
            self.push_op(OpCode::BuildTuple(size));
        }

        // キーワード引数のデフォルト値の設定
        let has_kw_default = param.named_list.iter().any(|v| v.expr.is_some());
        if has_kw_default {
            let mut name_list: Vec<&str> = vec![];
            for v in &param.named_list {
                match &v.expr {
                    Some(expr) => {
                        self.compile_expr(expr)?;
                        name_list.push(v.identifier.value);
                    }
                    None => (),
                }
            }
            self.push_load_const(PyObject::SmallTuple {
                children: name_list
                    .iter()
                    .map(|v| PyObject::new_string(v.to_string(), false))
                    .collect(),
                add_ref: false,
            });
            let size = name_list.len() as u8;
            self.push_op(OpCode::BuildConstKeyMap(size));
        }

        // アノテーションは未実装
        // クロージャは未実装

        // コードオブジェクトの読み込み
        self.push_load_const(py_code);
        // 関数名の読み込み
        match function_name_prefix {
            Some(prefix) => {
                self.push_load_const(PyObject::new_string(format!("{}{}", prefix, name), false));
            }
            None => {
                self.push_load_const(PyObject::new_string(name.to_string(), false));
            }
        }
        // 関数作成と収納
        let make_flag = (has_default as u8) | ((has_kw_default as u8) << 1);
        self.push_op(OpCode::MakeFunction(make_flag));
        let p = (**self.context_stack.last().unwrap())
            .borrow_mut()
            .declare_variable(&name);
        self.push_op(OpCode::StoreName(p));
        Ok(())
    }

    fn push_op(&self, op: OpCode) {
        self.byte_operations.borrow_mut().push(op);
    }

    fn push_load_const(&self, value: PyObject) -> u8 {
        let position = self.context_stack.last().unwrap().borrow().const_len() as u8;
        (**self.context_stack.last().unwrap())
            .borrow_mut()
            .push_const(value);
        self.push_op(OpCode::LoadConst(position));
        position
    }

    fn push_load_var(&self, value: &String) {
        let scope = self
            .context_stack
            .last()
            .unwrap()
            .borrow()
            .check_variable_scope(value);
        match scope {
            VariableScope::Global | VariableScope::NotDefined => {
                if self.context_stack.last().unwrap().borrow().is_global() {
                    let p = (**self.context_stack.last().unwrap())
                        .borrow_mut()
                        .register_or_get_name(value);
                    self.push_op(OpCode::LoadName(p));
                } else {
                    let p = (**self.context_stack.last().unwrap())
                        .borrow_mut()
                        .register_or_get_name(value);
                    self.push_op(OpCode::LoadGlobal(p));
                }
            }
            VariableScope::Local => {
                let p = self
                    .context_stack
                    .last()
                    .unwrap()
                    .borrow()
                    .get_local_variable(value);
                self.push_op(OpCode::LoadFast(p));
            }
            VariableScope::Instance => {
                let p = self
                    .context_stack
                    .last()
                    .unwrap()
                    .borrow()
                    .get_local_variable(&"self".to_string());
                self.push_op(OpCode::LoadFast(p));
                let p = (**self.context_stack.last().unwrap())
                    .borrow_mut()
                    .register_or_get_name(value);
                self.push_op(OpCode::LoadAttr(p));
            }
        }
    }

    fn push_store_var(&self, value: &String) {
        let scope = self
            .context_stack
            .last()
            .unwrap()
            .borrow()
            .check_variable_scope(value);
        match scope {
            VariableScope::Global | VariableScope::NotDefined => {
                if self.context_stack.last().unwrap().borrow().is_global() {
                    let p = (**self.context_stack.last().unwrap())
                        .borrow_mut()
                        .register_or_get_name(value);
                    self.push_op(OpCode::StoreName(p));
                } else {
                    let p = (**self.context_stack.last().unwrap())
                        .borrow_mut()
                        .register_or_get_name(value);
                    self.push_op(OpCode::StoreGlobal(p));
                }
            }
            VariableScope::Local => {
                let p = self
                    .context_stack
                    .last()
                    .unwrap()
                    .borrow()
                    .get_local_variable(value);
                self.push_op(OpCode::StoreFast(p));
            }
            VariableScope::Instance => {
                let p = self
                    .context_stack
                    .last()
                    .unwrap()
                    .borrow()
                    .get_local_variable(&"self".to_string());
                self.push_op(OpCode::LoadFast(p));
                let p = (**self.context_stack.last().unwrap())
                    .borrow_mut()
                    .register_or_get_name(value);
                self.push_op(OpCode::StoreAttr(p));
            }
        }
    }

    fn gen_jump_label(&self) -> u32 {
        let key = *self.jump_label_key_index.borrow();
        *self.jump_label_key_index.borrow_mut() += 1;
        key
    }

    // 今の次の位置にラベル位置を合わせる
    fn set_jump_label_value(&self, key: u32) {
        let index = self.byte_operations.borrow().len() as u8;
        // 1命令あたり2バイトなので2倍
        self.jump_label_table.borrow_mut().insert(key, index * 2);
    }

    fn set_jump_label_value_offset(&self, key: u32, offset: u8) {
        let index = (self.byte_operations.borrow().len() as u8) - offset;
        // 1命令あたり2バイトなので2倍
        self.jump_label_table.borrow_mut().insert(key, index * 2);
    }
}

impl<'a, 'b> ByteCompiler<'a, 'b> {
    pub fn resolve_references(&self) -> Vec<ByteCode> {
        let opcode_list = self.byte_operations.borrow();

        let result = opcode_list
            .iter()
            .map(|v| v.resolve(&self.jump_label_table.borrow()))
            .collect();

        result
    }
}

fn replace_escape(source: &str) -> Result<String> {
    if !source.contains("\\") {
        return Ok(source.to_string());
    }

    let mut result = String::new();
    let mut chars = source.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next().unwrap() {
                'n' => result.push('\x0A'), // newline
                'r' => result.push('\x0D'), // carriage return
                'f' => result.push('\x0C'), // form feed
                'b' => result.push('\x08'), // backspace
                't' => result.push('\x09'), // tab
                'v' => result.push('\x0B'), // vertical tab
                'x' => {
                    // \xXX is equivalent to \u{XX}
                    let mut hex = String::new();
                    hex.push(chars.next().unwrap());
                    hex.push(chars.next().unwrap());
                    result.push(unicode_hex_to_char(&hex));
                }
                'u' => {
                    // \uXXXX or \u{X} or \u{XX} or \u{XXX} or \u{XXXX} or \u{XXXXX} or \u{XXXXXX}
                    match chars.next().unwrap() {
                        '{' => {
                            let mut hex = String::new();
                            let mut count = 0;
                            loop {
                                let c = chars.next().unwrap();
                                if c == '}' {
                                    break;
                                }
                                hex.push(c);
                                count += 1;
                                if count > 6 {
                                    bail!("Invalid unicode escape sequence");
                                }
                            }
                            result.push(unicode_hex_to_char(&hex));
                        }
                        c => {
                            let mut hex = String::new();
                            hex.push(c);
                            hex.push(chars.next().unwrap());
                            hex.push(chars.next().unwrap());
                            hex.push(chars.next().unwrap());
                            result.push(unicode_hex_to_char(&hex));
                        }
                    }
                }
                c => result.push(c),
            }
        } else {
            result.push(c);
        }
    }

    Ok(result)
}

fn unicode_hex_to_char(hex: &String) -> char {
    let hex = u32::from_str_radix(hex, 16).unwrap();
    std::char::from_u32(hex).unwrap()
}
