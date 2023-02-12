use std::{cell::RefCell, collections::HashMap};

use cfgrammar::Span;

use crate::bytecode::{calc_stack_size, ByteCode};
use crate::executioncontext::{
    BlockContext, ExecutionContext, GlobalContext, PyContext, VariableScope,
};
use crate::{bytecode::OpCode, parser::Node, pyobject::PyObject};

pub struct ByteCompiler<'ctx, 'value> {
    pub byte_operations: RefCell<Vec<OpCode>>,
    context: &'ctx mut dyn ExecutionContext<'value>,
    jump_label_table: RefCell<HashMap<u32, u8>>,
    jump_label_key_index: RefCell<u32>,
    source: &'value str,
}

const PREDEFINED_VARIABLES: [&'static str; 1] = ["print"];

pub fn run_root<'value>(
    file_name: &'value str,
    root_node_list: &'value Vec<Box<Node>>,
    source: &'value str,
) -> PyObject<'value> {
    let mut global_context = GlobalContext {
        constant_list: vec![],
        name_list: vec![],
        name_map: HashMap::new(),
        global_variables: Vec::from(PREDEFINED_VARIABLES),
    };

    let mut compiler = ByteCompiler {
        byte_operations: RefCell::new(vec![]),
        context: &mut global_context,
        jump_label_table: RefCell::new(HashMap::new()),
        jump_label_key_index: RefCell::new(0),
        source: source,
    };

    compiler.context.push_const(PyObject::None(false));

    for node in root_node_list {
        compiler.compile(node);
    }

    // main関数を実行
    let main_position = compiler.context.register_or_get_name("main");
    compiler
        .byte_operations
        .borrow_mut()
        .push(OpCode::LoadName(main_position));
    compiler
        .byte_operations
        .borrow_mut()
        .push(OpCode::CallFunction(0));
    compiler.byte_operations.borrow_mut().push(OpCode::PopTop);
    compiler
        .byte_operations
        .borrow_mut()
        .push(OpCode::LoadConst(0));
    compiler
        .byte_operations
        .borrow_mut()
        .push(OpCode::ReturnValue);

    let stack_size = calc_stack_size(&compiler.byte_operations.borrow()) as u32;
    let operation_list = compiler.resolve_references();

    let constant_list = PyObject::SmallTuple {
        children: global_context.constant_list,
        add_ref: false,
    };
    let name_list = PyObject::SmallTuple {
        children: global_context.name_list,
        add_ref: false,
    };

    PyObject::Code {
        file_name: file_name,
        code_name: "<module>",
        num_locals: 0,
        stack_size: stack_size,
        operation_list: operation_list,
        constant_list: Box::new(constant_list),
        name_list: Box::new(name_list),
        local_list: Box::new(PyObject::SmallTuple { children: vec![], add_ref: false }),
        add_ref: true,
    }
}

pub fn run_function<'ctx, 'value, 'cpl>(
    file_name: &'value str,
    code_name: &'value str,
    outer_compiler: &'cpl ByteCompiler<'ctx, 'value>,
    body: &'value Node,
    source: &'value str,
) -> PyObject<'value> {
    let mut py_context = PyContext {
        outer: outer_compiler.context,
        constant_list: vec![],
        name_list: vec![],
        name_map: HashMap::new(),
        local_variables: vec![],
    };

    let mut block_context = BlockContext {
        outer: &mut py_context,
        variables: vec![],
    };

    let mut compiler = ByteCompiler {
        byte_operations: RefCell::new(vec![]),
        context: &mut block_context,
        jump_label_table: RefCell::new(HashMap::new()),
        jump_label_key_index: RefCell::new(*outer_compiler.jump_label_key_index.borrow()),
        source: source,
    };

    compiler.compile(body);

    // compiler.context_stack.borrow_mut().pop();
    // compiler.context_stack.borrow_mut().pop();

    let none_position = compiler.context.const_len() as u8;
    compiler.context.push_const(PyObject::None(false));
    compiler
        .byte_operations
        .borrow_mut()
        .push(OpCode::LoadConst(none_position));
    compiler
        .byte_operations
        .borrow_mut()
        .push(OpCode::ReturnValue);

    // outer_compilerへの情報の復帰
    *outer_compiler.jump_label_key_index.borrow_mut() = *compiler.jump_label_key_index.borrow();

    // PyCodeの作成
    let stack_size = calc_stack_size(&compiler.byte_operations.borrow()) as u32;
    let operation_list = compiler.resolve_references();

    PyObject::Code {
        file_name,
        code_name,
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
        local_list: Box::new(PyObject::SmallTuple { children: py_context
            .local_variables
            .iter()
            .map(|v| PyObject::new_string(v, false))
            .collect(), add_ref: false }),
        add_ref: false,
    }
}

impl<'ctx, 'value> ByteCompiler<'ctx, 'value> {
    fn compile(&mut self, node: &'value Node) {
        match node {
            Node::BinaryExpression {
                span: _,
                operator,
                left,
                right,
            } => {
                self.compile(left);
                self.compile(right);
                match *operator {
                    "==" | "!=" | ">=" | ">" | "<=" | "<" => self
                        .byte_operations
                        .borrow_mut()
                        .push(OpCode::compare_op_from_str(operator)),
                    "<<" => self.byte_operations.borrow_mut().push(OpCode::BinaryLShift),
                    ">>" => self.byte_operations.borrow_mut().push(OpCode::BinaryRShift),
                    "&" => self.byte_operations.borrow_mut().push(OpCode::BinaryAnd),
                    "^" => self.byte_operations.borrow_mut().push(OpCode::BinaryXor),
                    "|" => self.byte_operations.borrow_mut().push(OpCode::BinaryOr),
                    "+" => self.byte_operations.borrow_mut().push(OpCode::BinaryAdd),
                    "-" => self
                        .byte_operations
                        .borrow_mut()
                        .push(OpCode::BinarySubtract),
                    "*" => self
                        .byte_operations
                        .borrow_mut()
                        .push(OpCode::BinaryMultiply),
                    "/" => self
                        .byte_operations
                        .borrow_mut()
                        .push(OpCode::BinaryTrueDivide),
                    _ => panic!("unknown operator: {}", *operator),
                }
            }
            Node::UnaryOpExpression {
                span: _,
                operator,
                child,
            } => {
                self.compile(child);
                match *operator {
                    "-" => self
                        .byte_operations
                        .borrow_mut()
                        .push(OpCode::UnaryNegative),
                    "!" => self.byte_operations.borrow_mut().push(OpCode::UnaryNot),
                    "~" => self.byte_operations.borrow_mut().push(OpCode::UnaryInvert),
                    _ => panic!("unknown unary operator: {}", *operator),
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
                            self.compile(right);
                            match self.context.check_variable_scope(value) {
                                VariableScope::Global => {
                                    if self.context.is_global() {
                                        let p = self.context.register_or_get_name(value);
                                        self.byte_operations
                                            .borrow_mut()
                                            .push(OpCode::StoreName(p));
                                    } else {
                                        let p = self.context.register_or_get_name(value);
                                        self.byte_operations
                                            .borrow_mut()
                                            .push(OpCode::StoreGlobal(p));
                                    }
                                }
                                VariableScope::Local => {
                                    let p = self.context.register_or_get_name(value);
                                    self.byte_operations.borrow_mut().push(OpCode::StoreFast(p));
                                }
                                VariableScope::NotDefined => {
                                    panic!("{} is used before its declaration.", value);
                                }
                            }
                        }
                        "*=" | "/=" | "~/=" | "%=" | "+=" | "-=" | "<<=" | ">>=" | "&=" | "^="
                        | "|=" => {
                            let p = self.context.register_or_get_name(value);
                            let scope = self.context.check_variable_scope(value);
                            match scope {
                                VariableScope::Global => {
                                    if self.context.is_global() {
                                        self.byte_operations.borrow_mut().push(OpCode::LoadName(p));
                                    } else {
                                        self.byte_operations
                                            .borrow_mut()
                                            .push(OpCode::LoadGlobal(p));
                                    }
                                }
                                VariableScope::Local => {
                                    self.byte_operations.borrow_mut().push(OpCode::LoadFast(p));
                                }
                                VariableScope::NotDefined => {
                                    panic!("{} is used before its declaration.", value);
                                }
                            }
                            self.compile(right);
                            match *operator {
                                "*=" => self
                                    .byte_operations
                                    .borrow_mut()
                                    .push(OpCode::InplaceMultiply),
                                "/=" => self
                                    .byte_operations
                                    .borrow_mut()
                                    .push(OpCode::InplaceTrueDivide),
                                "~/=" => self
                                    .byte_operations
                                    .borrow_mut()
                                    .push(OpCode::InplaceFloorDivide),
                                "%=" => self
                                    .byte_operations
                                    .borrow_mut()
                                    .push(OpCode::InplaceModulo),
                                "+=" => self.byte_operations.borrow_mut().push(OpCode::InplaceAdd),
                                "-=" => self
                                    .byte_operations
                                    .borrow_mut()
                                    .push(OpCode::InplaceSubtract),
                                "<<=" => self
                                    .byte_operations
                                    .borrow_mut()
                                    .push(OpCode::InplaceLShift),
                                ">>=" => self
                                    .byte_operations
                                    .borrow_mut()
                                    .push(OpCode::InplaceRShift),
                                "&=" => self.byte_operations.borrow_mut().push(OpCode::InplaceAnd),
                                "^=" => self.byte_operations.borrow_mut().push(OpCode::InplaceXor),
                                "|=" => self.byte_operations.borrow_mut().push(OpCode::InplaceOr),
                                _ => (),
                            }
                            match scope {
                                VariableScope::Global => {
                                    if self.context.is_global() {
                                        self.byte_operations
                                            .borrow_mut()
                                            .push(OpCode::StoreName(p));
                                    } else {
                                        self.byte_operations
                                            .borrow_mut()
                                            .push(OpCode::StoreGlobal(p));
                                    }
                                }
                                VariableScope::Local => {
                                    self.byte_operations.borrow_mut().push(OpCode::StoreFast(p));
                                }
                                VariableScope::NotDefined => {
                                    panic!("{} is used before its declaration.", value);
                                }
                            }
                        }
                        "??=" => {
                            let p = self.context.register_or_get_name(value);
                            let scope = self.context.check_variable_scope(value);
                            match scope {
                                VariableScope::Global => {
                                    if self.context.is_global() {
                                        self.byte_operations.borrow_mut().push(OpCode::LoadName(p));
                                        let none_position = self.context.const_len() as u8;
                                        self.context.push_const(PyObject::None(false));
                                        self.byte_operations
                                            .borrow_mut()
                                            .push(OpCode::LoadConst(none_position));
                                        self.byte_operations
                                            .borrow_mut()
                                            .push(OpCode::compare_op_from_str("=="));
                                        let label_end = self.gen_jump_label();
                                        self.byte_operations
                                            .borrow_mut()
                                            .push(OpCode::PopJumpIfFalse(label_end));
                                        self.compile(right);
                                        self.byte_operations
                                            .borrow_mut()
                                            .push(OpCode::StoreName(p));
                                        self.set_jump_label_value(label_end);
                                    } else {
                                        self.byte_operations
                                            .borrow_mut()
                                            .push(OpCode::LoadGlobal(p));
                                        let none_position = self.context.const_len() as u8;
                                        self.context.push_const(PyObject::None(false));
                                        self.byte_operations
                                            .borrow_mut()
                                            .push(OpCode::LoadConst(none_position));
                                        self.byte_operations
                                            .borrow_mut()
                                            .push(OpCode::compare_op_from_str("=="));
                                        let label_end = self.gen_jump_label();
                                        self.byte_operations
                                            .borrow_mut()
                                            .push(OpCode::PopJumpIfFalse(label_end));
                                        self.compile(right);
                                        self.byte_operations
                                            .borrow_mut()
                                            .push(OpCode::StoreGlobal(p));
                                        self.set_jump_label_value(label_end);
                                    }
                                }
                                VariableScope::Local => {
                                    self.byte_operations.borrow_mut().push(OpCode::LoadFast(p));
                                    let none_position = self.context.const_len() as u8;
                                    self.context.push_const(PyObject::None(false));
                                    self.byte_operations
                                        .borrow_mut()
                                        .push(OpCode::LoadConst(none_position));
                                    self.byte_operations
                                        .borrow_mut()
                                        .push(OpCode::compare_op_from_str("=="));
                                    let label_end = self.gen_jump_label();
                                    self.byte_operations
                                        .borrow_mut()
                                        .push(OpCode::PopJumpIfFalse(label_end));
                                    self.compile(right);
                                    self.byte_operations.borrow_mut().push(OpCode::StoreFast(p));
                                    self.set_jump_label_value(label_end);
                                }
                                VariableScope::NotDefined => {
                                    panic!("{} is used before its declaration.", value);
                                }
                            }
                        }
                        _ => panic!("Unknown assignment operator: {}", value),
                    }
                    // Expressionはスタックになにか残しておきたいので積む
                    self.byte_operations.borrow_mut().push(OpCode::LoadConst(0));
                } else {
                    panic!("Invalid AST. Assignment lhs must be an identifier.");
                }
            }
            Node::NumericLiteral { span } => {
                let const_position = self.context.const_len() as u8;
                let raw_value = self.span_to_str(span);
                self.context
                    .push_const(PyObject::new_numeric(raw_value, false));
                self.byte_operations
                    .borrow_mut()
                    .push(OpCode::LoadConst(const_position));
            }
            Node::StringLiteral { span } => {
                // let value = self.span_to_str(span);
                let value = &self.source[span.start()..span.end()];
                let len = value.len();
                let value = &value[1..len - 1];

                let const_position = self.context.const_len() as u8;
                self.context.push_const(PyObject::new_string(value, false));
                self.byte_operations
                    .borrow_mut()
                    .push(OpCode::LoadConst(const_position));
            }
            Node::BooleanLiteral { span } => {
                let value = self.span_to_str(span);
                let const_position = self.context.const_len() as u8;
                self.context.push_const(PyObject::new_boolean(value, false));
                self.byte_operations
                    .borrow_mut()
                    .push(OpCode::LoadConst(const_position));
            }
            Node::NullLiteral { span: _ } => {
                let const_position = self.context.const_len() as u8;
                self.context.push_const(PyObject::None(false));
                self.byte_operations
                    .borrow_mut()
                    .push(OpCode::LoadConst(const_position));
            }
            Node::Identifier { span } => {
                let value = self.span_to_str(span);
                match self.context.check_variable_scope(value) {
                    VariableScope::Global => {
                        if self.context.is_global() {
                            let p = self.context.register_or_get_name(value);
                            self.byte_operations.borrow_mut().push(OpCode::LoadName(p));
                        } else {
                            let p = self.context.register_or_get_name(value);
                            self.byte_operations
                                .borrow_mut()
                                .push(OpCode::LoadGlobal(p));
                        }
                    }
                    VariableScope::Local => {
                        let p = self.context.register_or_get_name(value);
                        self.byte_operations.borrow_mut().push(OpCode::LoadFast(p));
                    }
                    VariableScope::NotDefined => {
                        panic!("{} is used before its declaration.", value);
                    }
                }
            }
            Node::Arguments { span: _, children } => {
                for node in children {
                    self.compile(node);
                }
                self.byte_operations
                    .borrow_mut()
                    .push(OpCode::CallFunction(children.len() as u8))
            }
            Node::WithSelectorExpression {
                span: _,
                child,
                selector,
            } => {
                self.compile(child);
                self.compile(selector);
            }

            Node::EmptyStatement { span: _ } => {}
            Node::ExpressionStatement { span: _, expr } => {
                self.compile(expr);
                self.byte_operations.borrow_mut().push(OpCode::PopTop);
            }
            Node::BlockStatement { span: _, children } => {
                for child in children {
                    self.compile(child);
                }
            }

            Node::VariableDeclaration {
                span: _,
                identifier,
                expr,
            } => match expr {
                Some(e) => {
                    self.compile(e);
                    if let Node::Identifier { span: id_span } = **identifier {
                        let value = self.span_to_str(&id_span);
                        let position = self.context.declare_variable(value);
                        if self.context.is_global() {
                            // トップレベル変数の場合
                            self.byte_operations
                                .borrow_mut()
                                .push(OpCode::StoreName(position));
                        } else {
                            // ローカル変数の場合
                            self.byte_operations
                                .borrow_mut()
                                .push(OpCode::StoreFast(position));
                        }
                    } else {
                        panic!("Invalid AST");
                    }
                }
                None => {
                    if let Node::Identifier { span: id_span } = **identifier {
                        let value = &self.span_to_str(&id_span);
                        self.context.declare_variable(value);
                    } else {
                        panic!("Invalid AST");
                    }
                }
            },
            Node::FunctionDeclaration {
                span: _,
                identifier,
                parameters: _,
                body,
            } => {
                if let Node::Identifier { span } = **identifier {
                    let name = self.span_to_str(&span);
                    // TODO: parametersの利用
                    let py_code = run_function("main.py", name, self, body, self.source);

                    // コードオブジェクトの読み込み
                    let position = self.context.const_len() as u8;
                    self.context.push_const(py_code);
                    self.byte_operations
                        .borrow_mut()
                        .push(OpCode::LoadConst(position));

                    // 関数名の読み込み
                    let position = self.context.const_len() as u8;
                    self.context.push_const(PyObject::new_string(name, false));
                    self.byte_operations
                        .borrow_mut()
                        .push(OpCode::LoadConst(position));

                    // 関数作成と収納
                    self.byte_operations.borrow_mut().push(OpCode::MakeFunction);
                    let p = self.context.declare_variable(name);
                    self.byte_operations.borrow_mut().push(OpCode::StoreName(p));
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
                        self.compile(condition);
                        let label_false_starts = self.gen_jump_label();
                        self.byte_operations
                            .borrow_mut()
                            .push(OpCode::PopJumpIfFalse(label_false_starts));
                        self.compile(if_true_stmt);

                        let label_if_ends = self.gen_jump_label();
                        self.byte_operations
                            .borrow_mut()
                            .push(OpCode::JumpAbsolute(label_if_ends));

                        self.set_jump_label_value(label_false_starts);
                        self.compile(if_false_stmt);

                        self.set_jump_label_value(label_if_ends);
                    }
                    None => {
                        // if expr stmt
                        self.compile(condition);
                        let label_if_ends = self.gen_jump_label();
                        self.byte_operations
                            .borrow_mut()
                            .push(OpCode::PopJumpIfFalse(label_if_ends));
                        self.compile(if_true_stmt);

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
                if let Some(node) = init {
                    self.compile(node);
                }
                self.set_jump_label_value(label_loop_start);
                if let Some(node) = condition {
                    self.compile(node);
                    self.byte_operations
                        .borrow_mut()
                        .push(OpCode::PopJumpIfFalse(label_for_end));
                }
                self.compile(stmt);
                if let Some(node_list) = update {
                    for node in node_list {
                        self.compile(node);
                        self.byte_operations.borrow_mut().push(OpCode::PopTop);
                    }
                }
                self.byte_operations
                    .borrow_mut()
                    .push(OpCode::JumpAbsolute(label_loop_start));
                self.set_jump_label_value(label_for_end);
            }
            Node::WhileStatement {
                span: _,
                condition,
                stmt,
            } => {
                let label_while_end = self.gen_jump_label();
                let label_loop_start = self.gen_jump_label();
                self.set_jump_label_value(label_loop_start);
                self.compile(condition);
                self.byte_operations
                    .borrow_mut()
                    .push(OpCode::PopJumpIfFalse(label_while_end));

                self.compile(stmt);
                self.byte_operations
                    .borrow_mut()
                    .push(OpCode::JumpAbsolute(label_loop_start));

                self.set_jump_label_value(label_while_end);
            }
            Node::DoStatement {
                span: _,
                condition,
                stmt,
            } => {
                let label_do_start = self.gen_jump_label();
                self.set_jump_label_value(label_do_start);
                self.compile(stmt);

                self.compile(condition);
                self.byte_operations
                    .borrow_mut()
                    .push(OpCode::PopJumpIfTrue(label_do_start));
            }
        }
    }

    // fn context(&self) -> &'a Box<dyn ExecutionContext> {
    //     return self.context_stack.borrow().last().unwrap();
    // }

    fn span_to_str(&self, span: &Span) -> &'value str {
        return &self.source[span.start()..span.end()];
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
