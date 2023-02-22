use std::rc::Rc;
use std::{cell::RefCell, collections::HashMap};

use cfgrammar::Span;

use crate::bytecode::{calc_stack_size, ByteCode};
use crate::executioncontext::{
    BlockContext, ExecutionContext, GlobalContext, PyContext, VariableScope,
};
use crate::parser::{LibraryDeclaration, LibraryImport};
use crate::{bytecode::OpCode, parser::Node, pyobject::PyObject};

struct DefaultScope {
    break_label: u32,
    continue_label: Option<u32>,
}

pub struct ByteCompiler<'ctx, 'value: 'ctx> {
    pub byte_operations: RefCell<Vec<OpCode>>,
    context_stack: Vec<Rc<RefCell<dyn ExecutionContext<'value> + 'ctx>>>,
    jump_label_table: RefCell<HashMap<u32, u8>>,
    jump_label_key_index: RefCell<u32>,
    default_scope_stack: Vec<DefaultScope>,
    break_label_table: HashMap<&'value str, u32>,
    continue_label_table: HashMap<&'value str, u32>,
    source: &'value str,
}

const PREDEFINED_VARIABLES: [&'static str; 1] = ["print"];

pub fn run_root<'value>(
    file_name: &'value str,
    root_node: &'value LibraryDeclaration,
    source: &'value str,
) -> PyObject {
    let global_context = Rc::new(RefCell::new(GlobalContext {
        constant_list: vec![],
        name_list: vec![],
        name_map: HashMap::new(),
        global_variables: Vec::from(PREDEFINED_VARIABLES),
    }));

    let mut compiler = ByteCompiler {
        byte_operations: RefCell::new(vec![]),
        context_stack: vec![global_context.clone()],
        jump_label_table: RefCell::new(HashMap::new()),
        jump_label_key_index: RefCell::new(0),
        default_scope_stack: vec![],
        break_label_table: HashMap::new(),
        continue_label_table: HashMap::new(),
        source,
    };

    // 0番目の定数にNoneを追加
    (*global_context)
        .borrow_mut()
        .push_const(PyObject::None(false));

    for node in &root_node.import_list {
        compiler.compile_import(node);
    }

    for node in &root_node.top_level_declaration_list {
        compiler.compile(&node, None);
    }

    // main関数を実行
    let main_position = (*global_context)
        .borrow_mut()
        .register_or_get_name("main".to_string());
    compiler.push_op(OpCode::LoadName(main_position));
    compiler.push_op(OpCode::CallFunction(0));
    compiler.push_op(OpCode::PopTop);
    compiler.push_op(OpCode::LoadConst(0));
    compiler.push_op(OpCode::ReturnValue);

    let stack_size = calc_stack_size(&compiler.byte_operations.borrow()) as u32;
    let operation_list = compiler.resolve_references();

    compiler.context_stack.pop();

    // 所有が1箇所しかないはずなのでRcの外に出す
    let global_context = Rc::try_unwrap(global_context).ok().unwrap().into_inner();

    let constant_list = PyObject::SmallTuple {
        children: global_context.constant_list,
        add_ref: false,
    };
    let name_list = PyObject::SmallTuple {
        children: global_context.name_list,
        add_ref: false,
    };

    PyObject::Code {
        file_name: file_name.to_string(),
        code_name: "<module>".to_string(),
        num_args: 0,
        num_locals: 0,
        stack_size,
        operation_list,
        constant_list: Box::new(constant_list),
        name_list: Box::new(name_list),
        local_list: Box::new(PyObject::SmallTuple {
            children: vec![],
            add_ref: false,
        }),
        add_ref: true,
    }
}

pub fn run_function<'ctx, 'value, 'cpl>(
    file_name: &'value str,
    code_name: &'value str,
    argument_list: Vec<&'value str>,
    outer_compiler: &'cpl ByteCompiler<'ctx, 'value>,
    body: &'value Node,
    source: &'value str,
) -> PyObject {
    let py_context = Rc::new(RefCell::new(PyContext {
        outer: outer_compiler.context_stack.last().unwrap().clone(),
        constant_list: vec![],
        name_list: vec![],
        name_map: HashMap::new(),
        local_variables: vec![],
    }));

    let block_context = Rc::new(RefCell::new(BlockContext {
        outer: py_context.clone(),
        variables: vec![],
    }));

    let num_args = argument_list.len() as u32;
    for arg in argument_list {
        (*block_context).borrow_mut().declare_variable(arg);
    }

    let mut compiler = ByteCompiler {
        byte_operations: RefCell::new(vec![]),
        context_stack: vec![block_context.clone()],
        jump_label_table: RefCell::new(HashMap::new()),
        jump_label_key_index: RefCell::new(*outer_compiler.jump_label_key_index.borrow()),
        default_scope_stack: vec![],
        break_label_table: HashMap::new(),
        continue_label_table: HashMap::new(),
        source,
    };

    compiler.compile(body, None);

    compiler.push_load_const(PyObject::None(false));
    compiler.push_op(OpCode::ReturnValue);

    compiler.context_stack.pop();
    drop(block_context);

    // outer_compilerへの情報の復帰
    *outer_compiler.jump_label_key_index.borrow_mut() = *compiler.jump_label_key_index.borrow();

    // PyCodeの作成
    let stack_size = calc_stack_size(&compiler.byte_operations.borrow()) as u32;
    let operation_list = compiler.resolve_references();

    let py_context = Rc::try_unwrap(py_context).ok().unwrap().into_inner();

    PyObject::Code {
        file_name: file_name.to_string(),
        code_name: code_name.to_string(),
        num_args,
        num_locals: py_context.local_variables.len() as u32,
        stack_size,
        operation_list,
        constant_list: Box::new(PyObject::SmallTuple {
            children: py_context.constant_list,
            add_ref: false,
        }),
        name_list: Box::new(PyObject::SmallTuple {
            children: py_context.name_list,
            add_ref: false,
        }),
        local_list: Box::new(PyObject::SmallTuple {
            children: py_context
                .local_variables
                .iter()
                .map(|v| PyObject::new_string(v.to_string(), false))
                .collect(),
            add_ref: false,
        }),
        add_ref: false,
    }
}

impl<'ctx, 'value> ByteCompiler<'ctx, 'value> {
    fn compile_import(&mut self, node: &'value LibraryImport) {
        let uri = self.span_to_str(&node.uri);
        let len = uri.len();
        let uri = &uri[1..len - 1];

        let identifier = match &node.identifier {
            Some(v) => {
                if let Node::Identifier { span } = **v {
                    Some(self.span_to_str(&span))
                } else {
                    None
                }
            }
            None => None,
        };

        // uri形式
        // import A.B as C
        // → import "py:A/B" as C;
        // from ..A.B import C as D
        // → import "py:../A/B/C" as D;
        if !uri.starts_with("py:") {
            panic!("invalid import uri: {}", uri);
        }

        let splitted: Vec<&str> = uri.split(':').collect();
        if splitted.len() != 2 {
            panic!("invalid import uri: {}", uri);
        }
        let mut path_splitted: Vec<&str> = splitted[1].split("/").collect();

        if splitted[1].starts_with(".") {
            // 相対パスの場合

            // ドットの数を積む
            let dot_len = path_splitted[0].len();
            self.push_load_const(PyObject::Int(dot_len as i32, false));
            path_splitted.remove(0);

            // 最後尾のモジュールをタプルで積む
            let import_mod = path_splitted.pop().unwrap();
            let import_mod_p = self.push_load_const(PyObject::SmallTuple {
                    children: vec![PyObject::new_string(import_mod.to_string(), false)],
                    add_ref: false,
                });

            // 名前でインポート
            let p = (**self.context_stack.last().unwrap())
                .borrow_mut()
                .register_or_get_name(path_splitted.join("."));
            self.push_op(OpCode::ImportName(p));

            // インポート先のモジュール
            self.push_op(OpCode::ImportFrom(import_mod_p));

            // 格納先
            let store_name = match identifier {
                Some(v) => v,
                None => import_mod,
            };
            let store_name_p = (**self.context_stack.last().unwrap())
                .borrow_mut()
                .declare_variable(store_name);
            self.push_op(OpCode::StoreName(store_name_p));
            self.push_op(OpCode::PopTop);
        } else {
            // 0を積む
            self.push_load_const(PyObject::Int(0, false));

            // Noneを積む
            self.push_load_const(PyObject::None(false));

            // 名前でインポート
            let import_name = path_splitted.join(".");
            let import_name_p = (**self.context_stack.last().unwrap())
                .borrow_mut()
                .register_or_get_name(import_name);
            self.push_op(OpCode::ImportName(import_name_p));

            match identifier {
                None => {
                    // import A.B
                    let store_name = path_splitted[0];
                    let store_name_p = (**self.context_stack.last().unwrap())
                        .borrow_mut()
                        .declare_variable(store_name);
                    self.push_op(OpCode::StoreName(store_name_p));
                }
                Some(v) => {
                    if path_splitted.len() == 1 {
                        // import A as B
                        let p = (**self.context_stack.last().unwrap())
                            .borrow_mut()
                            .declare_variable(v);
                        self.push_op(OpCode::StoreName(p));
                    } else {
                        // import A.B.C as D
                        let second_name = path_splitted[1].to_string();
                        let p = (**self.context_stack.last().unwrap())
                            .borrow_mut()
                            .register_or_get_name(second_name);
                        self.push_op(OpCode::ImportFrom(p));
                        for i in 2..path_splitted.len() {
                            self.push_op(OpCode::RotTwo);
                            self.push_op(OpCode::PopTop);
                            let p = (**self.context_stack.last().unwrap())
                                .borrow_mut()
                                .register_or_get_name(path_splitted[i].to_string());
                            self.push_op(OpCode::ImportFrom(p));
                        }
                        let p = (**self.context_stack.last().unwrap())
                            .borrow_mut()
                            .declare_variable(v);
                        self.push_op(OpCode::StoreName(p));
                        self.push_op(OpCode::PopTop);
                    }
                }
            }
        }
    }

    fn compile(&mut self, node: &'value Node, label: Option<&'value str>) {
        match node {
            Node::BinaryExpression {
                span: _,
                operator,
                left,
                right,
            } => {
                if *operator == "??" {
                    self.compile(left, None);
                    self.push_op(OpCode::DupTop);
                    self.push_load_const(PyObject::None(false));
                    self.push_op(OpCode::compare_op_from_str("=="));
                    let label_end = self.gen_jump_label();
                    self.push_op(OpCode::PopJumpIfFalse(label_end));
                    self.push_op(OpCode::PopTop);
                    self.compile(right, None);
                    self.set_jump_label_value(label_end);
                } else if *operator == "||" {
                    self.compile(left, None);
                    self.push_op(OpCode::DupTop);
                    let label_end = self.gen_jump_label();
                    self.push_op(OpCode::PopJumpIfTrue(label_end));
                    self.push_op(OpCode::PopTop);
                    self.compile(right, None);
                    self.set_jump_label_value(label_end);
                } else if *operator == "&&" {
                    self.compile(left, None);
                    self.push_op(OpCode::DupTop);
                    let label_end = self.gen_jump_label();
                    self.push_op(OpCode::PopJumpIfFalse(label_end));
                    self.push_op(OpCode::PopTop);
                    self.compile(right, None);
                    self.set_jump_label_value(label_end);
                } else {
                    self.compile(left, None);
                    self.compile(right, None);
                    match *operator {
                        "==" | "!=" | ">=" | ">" | "<=" | "<" => {
                            self.push_op(OpCode::compare_op_from_str(operator))
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
                        _ => panic!("unknown operator: {}", *operator),
                    }
                }
            }
            Node::ConditionalExpression {
                span: _,
                condition,
                if_true_expr,
                if_false_expr,
            } => {
                let label_conditional_end = self.gen_jump_label();
                let label_false_start = self.gen_jump_label();
                self.compile(condition, None);
                self.push_op(OpCode::PopJumpIfFalse(label_false_start));
                self.compile(if_true_expr, None);
                self.push_op(OpCode::JumpAbsolute(label_conditional_end));
                self.set_jump_label_value(label_false_start);
                self.compile(if_false_expr, None);
                self.set_jump_label_value(label_conditional_end);
            }
            Node::UnaryOpExpression {
                span: _,
                operator,
                child,
            } => {
                self.compile(child, None);
                match *operator {
                    "-" => self.push_op(OpCode::UnaryNegative),
                    "!" => self.push_op(OpCode::UnaryNot),
                    "~" => self.push_op(OpCode::UnaryInvert),
                    _ => panic!("unknown unary operator: {}", *operator),
                }
            }
            Node::UpdateExpression {
                span: _,
                operator,
                is_prefix,
                child,
            } => {
                if let Node::Identifier { span } = **child {
                    let value = self.span_to_str(&span);
                    if *is_prefix {
                        // 前置
                        self.push_load_var(value);
                        self.push_load_const(PyObject::Int(1, false));
                        match *operator {
                            "++" => self.push_op(OpCode::InplaceAdd),
                            "--" => self.push_op(OpCode::InplaceSubtract),
                            _ => (),
                        }
                        self.push_op(OpCode::DupTop);
                        self.push_store_var(value);
                    } else {
                        // 後置
                        self.push_load_var(value);
                        self.push_op(OpCode::DupTop);
                        self.push_load_const(PyObject::Int(1, false));
                        match *operator {
                            "++" => self.push_op(OpCode::InplaceAdd),
                            "--" => self.push_op(OpCode::InplaceSubtract),
                            _ => (),
                        }
                        self.push_store_var(value);
                    }
                } else {
                    panic!("Invalid AST. Increment target must be an identifier.");
                }
            }
            Node::AssignmentExpression {
                span: _,
                operator,
                left,
                right,
            } => {
                if let Node::Identifier { span } = **left {
                    let value = self.span_to_str(&span);
                    match *operator {
                        "=" => {
                            self.compile(right, None);
                            // DartではAssignment Expressionが代入先の最終的な値を残す
                            self.push_op(OpCode::DupTop);
                            let scope = self
                                .context_stack
                                .last()
                                .unwrap()
                                .borrow()
                                .check_variable_scope(value);
                            match scope {
                                VariableScope::Global => {
                                    if self.context_stack.last().unwrap().borrow().is_global() {
                                        let p = (**self.context_stack.last().unwrap())
                                            .borrow_mut()
                                            .register_or_get_name(value.to_string());
                                        self.push_op(OpCode::StoreName(p));
                                    } else {
                                        let p = (**self.context_stack.last().unwrap())
                                            .borrow_mut()
                                            .register_or_get_name(value.to_string());
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
                                VariableScope::NotDefined => {
                                    panic!("{} is used before its declaration.", value);
                                }
                            }
                        }
                        "*=" | "/=" | "~/=" | "%=" | "+=" | "-=" | "<<=" | ">>=" | "&=" | "^="
                        | "|=" => {
                            self.push_load_var(value);
                            self.compile(right, None);
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
                            self.push_store_var(value);
                        }
                        "??=" => {
                            self.push_load_var(value);
                            self.push_op(OpCode::DupTop);
                            self.push_load_const(PyObject::None(false));
                            self.push_op(OpCode::compare_op_from_str("=="));
                            let label_end = self.gen_jump_label();
                            self.push_op(OpCode::PopJumpIfFalse(label_end));
                            self.push_op(OpCode::PopTop);
                            self.compile(right, None);
                            self.push_op(OpCode::DupTop);
                            self.push_store_var(value);
                            self.set_jump_label_value(label_end);
                        }
                        _ => panic!("Unknown assignment operator: {}", value),
                    }
                } else {
                    panic!("Invalid AST. Assignment lhs must be an identifier.");
                }
            }
            Node::NumericLiteral { span } => {
                let raw_value = self.span_to_str(span);
                self.push_load_const(PyObject::new_numeric(raw_value, false));
            }
            Node::StringLiteral {
                span: _,
                literal_list,
            } => {
                let value = literal_list
                    .iter()
                    .map(|v| {
                        let len = v.len();
                        &self.span_to_str(v)[1..len - 1]
                    })
                    .collect::<Vec<&'value str>>()
                    .join("");

                self.push_load_const(PyObject::new_string(value.to_string(), false));
            }
            Node::BooleanLiteral { span } => {
                let value = self.span_to_str(span);
                self.push_load_const(PyObject::new_boolean(value, false));
            }
            Node::NullLiteral { span: _ } => {
                self.push_load_const(PyObject::None(false));
            }
            Node::Identifier { span } => {
                let value = self.span_to_str(span);
                self.push_load_var(value);
            }
            Node::SelectorAttr {
                span: _,
                identifier,
            } => {
                if let Node::Identifier { span } = **identifier {
                    let name = self.span_to_str(&span);
                    let p = (**self.context_stack.last().unwrap())
                        .borrow_mut()
                        .register_or_get_name(name.to_string());
                    self.push_op(OpCode::LoadAttr(p));
                } else {
                    panic!("Invalid AST");
                }
            }
            Node::SelectorMethod {
                span: _,
                identifier,
                arguments,
            } => {
                if let Node::Identifier { span } = **identifier {
                    let name = self.span_to_str(&span);
                    let p = (**self.context_stack.last().unwrap())
                        .borrow_mut()
                        .register_or_get_name(name.to_string());
                    self.push_op(OpCode::LoadMethod(p));

                    if let Node::Arguments { span: _, children } = &**arguments {
                        for node in children {
                            self.compile(node, None);
                        }
                        self.push_op(OpCode::CallMethod(children.len() as u8))
                    }
                } else {
                    panic!("Invalid AST");
                }
            }
            Node::Arguments { span: _, children } => {
                for node in children {
                    self.compile(node, None);
                }
                self.push_op(OpCode::CallFunction(children.len() as u8))
            }
            Node::WithSelectorExpression {
                span: _,
                child,
                selector,
            } => {
                self.compile(child, None);
                self.compile(selector, None);
            }

            Node::LabeledStatement {
                span: _,
                label,
                stmt,
            } => {
                let label_str = if let Node::Identifier { span } = *label.identifier {
                    self.span_to_str(&span)
                } else {
                    panic!("Invalid AST")
                };
                let label_id = self.gen_jump_label();

                // break用のラベルはこの時点で用意する
                self.break_label_table.insert(label_str, label_id);

                self.compile(stmt, Some(label_str));

                self.set_jump_label_value(label_id);
                self.break_label_table.remove(label_str);
            }
            Node::BreakStatement { span: _, label } => match label {
                Some(identifier) => {
                    if let Node::Identifier { span } = **identifier {
                        let label_str = self.span_to_str(&span);
                        match self.break_label_table.get(label_str) {
                            Some(v) => {
                                self.push_op(OpCode::JumpAbsolute(*v));
                            }
                            None => panic!("label {} is not existing in this scope.", label_str),
                        }
                    } else {
                        panic!("Invalid AST")
                    }
                }
                None => match self.default_scope_stack.last() {
                    Some(v) => {
                        self.push_op(OpCode::JumpAbsolute(v.break_label));
                    }
                    None => panic!("break statement is not available here"),
                },
            },
            Node::ContinueStatement { span: _, label } => match label {
                Some(identifier) => {
                    if let Node::Identifier { span } = **identifier {
                        let label_str = self.span_to_str(&span);
                        match self.continue_label_table.get(label_str) {
                            Some(v) => {
                                self.push_op(OpCode::JumpAbsolute(*v));
                            }
                            None => panic!("label {} is not existing in this scope.", label_str),
                        }
                    } else {
                        panic!("Invalid AST")
                    }
                }
                None => match self.default_scope_stack.last() {
                    Some(v) => match v.continue_label {
                        Some(continue_label) => self.push_op(OpCode::JumpAbsolute(continue_label)),
                        None => panic!("continue statement is not available here"),
                    },
                    None => panic!("continue statement is not available here"),
                },
            },
            Node::ReturnStatement { span: _, value } => {
                match value {
                    Some(v) => self.compile(v, None),
                    None => {
                        self.push_load_const(PyObject::None(false));
                    }
                }
                self.push_op(OpCode::ReturnValue);
            }
            Node::EmptyStatement { span: _ } => {}
            Node::ExpressionStatement { span: _, expr } => {
                self.compile(expr, None);
                self.push_op(OpCode::PopTop);
            }
            Node::BlockStatement { span: _, children } => {
                self.context_stack.push(Rc::new(RefCell::new(BlockContext {
                    outer: self.context_stack.last().unwrap().clone(),
                    variables: vec![],
                })));
                for child in children {
                    self.compile(child, None);
                }
                self.context_stack.pop();
            }

            Node::VariableDeclaration {
                span: _,
                identifier,
                expr,
            } => match expr {
                Some(e) => {
                    self.compile(e, None);
                    if let Node::Identifier { span: id_span } = **identifier {
                        let value = self.span_to_str(&id_span);
                        let position = (**self.context_stack.last().unwrap())
                            .borrow_mut()
                            .declare_variable(value);
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
                                .get_local_variable(value);
                            self.push_op(OpCode::StoreFast(local_position));
                        }
                    } else {
                        panic!("Invalid AST");
                    }
                }
                None => {
                    if let Node::Identifier { span: id_span } = **identifier {
                        let value = &self.span_to_str(&id_span);
                        (**self.context_stack.last().unwrap())
                            .borrow_mut()
                            .declare_variable(value);
                    } else {
                        panic!("Invalid AST");
                    }
                }
            },
            Node::FunctionDeclaration {
                span: _,
                identifier,
                parameters,
                body,
            } => {
                let mut argument_list: Vec<&str> = vec![];
                for p in parameters {
                    if let Node::Identifier { span } = *p.identifier {
                        let name = self.span_to_str(&span);
                        argument_list.push(name);
                    }
                }
                if let Node::Identifier { span } = **identifier {
                    let name = self.span_to_str(&span);
                    // TODO: parametersの利用
                    let py_code =
                        run_function("main.py", name, argument_list, self, body, self.source);

                    // コードオブジェクトの読み込み
                    let position = self.context_stack.last().unwrap().borrow().const_len() as u8;
                    (**self.context_stack.last().unwrap())
                        .borrow_mut()
                        .push_const(py_code);
                    self.push_op(OpCode::LoadConst(position));

                    // 関数名の読み込み
                    self.push_load_const(PyObject::new_string(name.to_string(), false));

                    // 関数作成と収納
                    self.push_op(OpCode::MakeFunction);
                    let p = (**self.context_stack.last().unwrap())
                        .borrow_mut()
                        .declare_variable(name);
                    self.push_op(OpCode::StoreName(p));
                } else {
                    panic!("Invalid AST");
                }
            }

            Node::IfStatement {
                span: _,
                condition,
                if_true_stmt,
                if_false_stmt,
            } => {
                match if_false_stmt {
                    Some(if_false_stmt) => {
                        // if expr stmt else stmt
                        self.compile(condition, None);
                        let label_false_starts = self.gen_jump_label();
                        self.push_op(OpCode::PopJumpIfFalse(label_false_starts));

                        self.context_stack.push(Rc::new(RefCell::new(BlockContext {
                            outer: self.context_stack.last().unwrap().clone(),
                            variables: vec![],
                        })));
                        self.compile(if_true_stmt, None);
                        self.context_stack.pop();

                        let label_if_ends = self.gen_jump_label();
                        self.push_op(OpCode::JumpAbsolute(label_if_ends));

                        self.set_jump_label_value(label_false_starts);

                        self.context_stack.push(Rc::new(RefCell::new(BlockContext {
                            outer: self.context_stack.last().unwrap().clone(),
                            variables: vec![],
                        })));
                        self.compile(if_false_stmt, None);
                        self.context_stack.pop();

                        self.set_jump_label_value(label_if_ends);
                    }
                    None => {
                        // if expr stmt
                        self.compile(condition, None);
                        let label_if_ends = self.gen_jump_label();
                        self.push_op(OpCode::PopJumpIfFalse(label_if_ends));
                        self.compile(if_true_stmt, None);

                        self.set_jump_label_value(label_if_ends);
                    }
                }
            }
            Node::ForStatement {
                span: _,
                init,
                condition,
                update,
                stmt,
            } => {
                // init is statement
                // condition is expression
                // update is expression list
                let label_for_end = self.gen_jump_label();
                let label_loop_start = self.gen_jump_label();
                self.default_scope_stack.push(DefaultScope {
                    break_label: label_for_end,
                    continue_label: Some(label_loop_start),
                });
                if let Some(stmt_label) = label {
                    self.continue_label_table
                        .insert(stmt_label, label_loop_start);
                }
                if let Some(node) = init {
                    self.compile(node, None);
                }
                self.set_jump_label_value(label_loop_start);
                if let Some(node) = condition {
                    self.compile(node, None);
                    self.push_op(OpCode::PopJumpIfFalse(label_for_end));
                }

                self.context_stack.push(Rc::new(RefCell::new(BlockContext {
                    outer: self.context_stack.last().unwrap().clone(),
                    variables: vec![],
                })));
                self.compile(stmt, None);
                self.context_stack.pop();

                if let Some(node_list) = update {
                    for node in node_list {
                        self.compile(node, None);
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
            Node::WhileStatement {
                span: _,
                condition,
                stmt,
            } => {
                let label_while_end = self.gen_jump_label();
                let label_loop_start = self.gen_jump_label();
                self.default_scope_stack.push(DefaultScope {
                    break_label: label_while_end,
                    continue_label: Some(label_loop_start),
                });
                if let Some(stmt_label) = label {
                    self.continue_label_table
                        .insert(stmt_label, label_loop_start);
                }
                self.set_jump_label_value(label_loop_start);
                self.compile(condition, None);
                self.push_op(OpCode::PopJumpIfFalse(label_while_end));

                self.context_stack.push(Rc::new(RefCell::new(BlockContext {
                    outer: self.context_stack.last().unwrap().clone(),
                    variables: vec![],
                })));
                self.compile(stmt, None);
                self.context_stack.pop();

                self.push_op(OpCode::JumpAbsolute(label_loop_start));

                self.set_jump_label_value(label_while_end);

                self.default_scope_stack.pop();
                if let Some(stmt_label) = label {
                    self.continue_label_table.remove(stmt_label);
                }
            }
            Node::DoStatement {
                span: _,
                condition,
                stmt,
            } => {
                let label_do_start = self.gen_jump_label();
                let label_do_end = self.gen_jump_label();
                self.default_scope_stack.push(DefaultScope {
                    break_label: label_do_end,
                    continue_label: Some(label_do_start),
                });
                if let Some(stmt_label) = label {
                    self.continue_label_table.insert(stmt_label, label_do_start);
                }
                self.set_jump_label_value(label_do_start);

                self.context_stack.push(Rc::new(RefCell::new(BlockContext {
                    outer: self.context_stack.last().unwrap().clone(),
                    variables: vec![],
                })));
                self.compile(stmt, None);
                self.context_stack.pop();

                self.compile(condition, None);
                self.push_op(OpCode::PopJumpIfTrue(label_do_start));
                self.set_jump_label_value(label_do_end);

                self.default_scope_stack.pop();
                if let Some(stmt_label) = label {
                    self.continue_label_table.remove(stmt_label);
                }
            }
            Node::SwitchStatement {
                span: _,
                expr,
                case_list,
                default_case,
            } => {
                self.compile(expr, None);
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
                    self.compile(&case.expr, None);
                    self.push_op(OpCode::compare_op_from_str("=="));
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
                        self.compile(stmt, None);
                    }
                }
                if let Some(default_case) = default_case {
                    self.set_jump_label_value(label_default_start);
                    for stmt in &default_case.stmt_list {
                        self.compile(stmt, None);
                    }
                }
                self.set_jump_label_value(label_switch_end);
                self.default_scope_stack.pop();
            }
        }
    }

    fn span_to_str(&self, span: &Span) -> &'value str {
        return &self.source[span.start()..span.end()];
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

    fn push_load_var(&self, value: &'value str) {
        let scope = self
            .context_stack
            .last()
            .unwrap()
            .borrow()
            .check_variable_scope(value);
        match scope {
            VariableScope::Global => {
                if self.context_stack.last().unwrap().borrow().is_global() {
                    let p = (**self.context_stack.last().unwrap())
                        .borrow_mut()
                        .register_or_get_name(value.to_string());
                    self.push_op(OpCode::LoadName(p));
                } else {
                    let p = (**self.context_stack.last().unwrap())
                        .borrow_mut()
                        .register_or_get_name(value.to_string());
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
            VariableScope::NotDefined => {
                panic!("{} is used before its declaration.", value);
            }
        }
    }

    fn push_store_var(&self, value: &'value str) {
        let scope = self
            .context_stack
            .last()
            .unwrap()
            .borrow()
            .check_variable_scope(value);
        match scope {
            VariableScope::Global => {
                if self.context_stack.last().unwrap().borrow().is_global() {
                    let p = (**self.context_stack.last().unwrap())
                        .borrow_mut()
                        .register_or_get_name(value.to_string());
                    self.push_op(OpCode::StoreName(p));
                } else {
                    let p = (**self.context_stack.last().unwrap())
                        .borrow_mut()
                        .register_or_get_name(value.to_string());
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
            VariableScope::NotDefined => {
                panic!("{} is used before its declaration.", value);
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
