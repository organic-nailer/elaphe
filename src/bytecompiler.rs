use std::{cell::RefCell, collections::HashMap};
use std::i32;

use cfgrammar::Span;

use crate::bytecode::ByteCode;
use crate::{bytecode::OpCode, parser::Node, pyobject::PyObject};

pub struct ByteCompiler<'a> {
    pub byte_operations: RefCell<Vec<OpCode>>,
    pub constant_list: RefCell<Vec<PyObject<'a>>>,
    pub name_list: RefCell<Vec<PyObject<'a>>>,
    pub name_map: RefCell<HashMap<&'a str, u8>>,
    pub global_variables: RefCell<Vec<&'a str>>,
    pub jump_label_table: RefCell<HashMap<u32, u8>>,
    jump_label_key_index: RefCell<u32>,
    source: &'a str,
}

const PREDEFINED_VARIABLES: [&'static str; 1] = ["print"];

impl ByteCompiler<'_> {
    pub fn run<'a>(root_node: &Node, source: &'a str) -> ByteCompiler<'a> {
        let compiler = ByteCompiler {
            byte_operations: RefCell::new(vec![]),
            constant_list: RefCell::new(vec![]),
            name_list: RefCell::new(vec![]),
            name_map: RefCell::new(HashMap::new()),
            global_variables: RefCell::new(Vec::from(PREDEFINED_VARIABLES)),
            jump_label_table: RefCell::new(HashMap::new()),
            jump_label_key_index: RefCell::new(0),
            source: source,
        };

        compiler.constant_list.borrow_mut().push(PyObject::None(false));

        compiler.compile(root_node);

        compiler
            .byte_operations
            .borrow_mut()
            .push(OpCode::LoadConst(0));
        compiler.byte_operations.borrow_mut().push(OpCode::ReturnValue);

        compiler
    }
}

impl<'a> ByteCompiler<'a> {
    fn compile(&self, node: &Node) {
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
                    "==" |
                    "!=" |
                    ">=" |
                    ">"  |
                    "<=" |
                    "<"  => self.byte_operations.borrow_mut().push(OpCode::compare_op_from_str(operator)),
                    "<<" => self.byte_operations.borrow_mut().push(OpCode::BinaryLShift),
                    ">>" => self.byte_operations.borrow_mut().push(OpCode::BinaryRShift),
                    "&" => self.byte_operations.borrow_mut().push(OpCode::BinaryAnd),
                    "^" => self.byte_operations.borrow_mut().push(OpCode::BinaryXor),
                    "|" => self.byte_operations.borrow_mut().push(OpCode::BinaryOr),
                    "+" => self.byte_operations.borrow_mut().push(OpCode::BinaryAdd),
                    "-" => self.byte_operations.borrow_mut().push(OpCode::BinarySubtract),
                    "*" => self.byte_operations.borrow_mut().push(OpCode::BinaryMultiply),
                    "/" => self.byte_operations.borrow_mut().push(OpCode::BinaryTrueDivide),
                    _ => panic!("unknown operator: {}", *operator),
                }
            },
            Node::UnaryOpExpression { span: _, operator, child } => {
                self.compile(child);
                match *operator {
                    "-" => self.byte_operations.borrow_mut().push(OpCode::UnaryNegative),
                    "!" => self.byte_operations.borrow_mut().push(OpCode::UnaryNot),
                    "~" => self.byte_operations.borrow_mut().push(OpCode::UnaryInvert),
                    _ => panic!("unknown unary operator: {}", *operator),
                }
            },
            Node::AssignmentExpression { span: _, operator, left, right } => {
                if let Node::Identifier { span } = **left {
                    let value = self.span_to_str(&span);
                    match *operator {
                        "=" => {
                            self.compile(right);
                            let position = self.search_name(value);
                            match position {
                                Some(v) => self.byte_operations.borrow_mut().push(OpCode::StoreName(v)),
                                None => panic!("{} is used before its declaration.", value),
                            }
                        },
                        "*=" |
                        "/=" |
                        "~/=" |
                        "%=" |
                        "+=" |
                        "-=" |
                        "<<=" |
                        ">>=" |
                        "&="|
                        "^=" |
                        "|=" => {
                            let position = self.search_name(value);
                            match position {
                                Some(v) => {
                                    self.byte_operations.borrow_mut().push(OpCode::LoadName(v));
                                    self.compile(right);
                                    match *operator {
                                        "*=" => self.byte_operations.borrow_mut().push(OpCode::InplaceMultiply),
                                        "/=" => self.byte_operations.borrow_mut().push(OpCode::InplaceTrueDivide),
                                        "~/=" => self.byte_operations.borrow_mut().push(OpCode::InplaceFloorDivide),
                                        "%=" => self.byte_operations.borrow_mut().push(OpCode::InplaceModulo),
                                        "+=" => self.byte_operations.borrow_mut().push(OpCode::InplaceAdd),
                                        "-=" => self.byte_operations.borrow_mut().push(OpCode::InplaceSubtract),
                                        "<<=" => self.byte_operations.borrow_mut().push(OpCode::InplaceLShift),
                                        ">>=" => self.byte_operations.borrow_mut().push(OpCode::InplaceRShift),
                                        "&=" => self.byte_operations.borrow_mut().push(OpCode::InplaceAnd),
                                        "^=" => self.byte_operations.borrow_mut().push(OpCode::InplaceXor),
                                        "|=" => self.byte_operations.borrow_mut().push(OpCode::InplaceOr),
                                        _ => ()
                                    }
                                    self.byte_operations.borrow_mut().push(OpCode::StoreName(v));
                                },
                                None => panic!("{} is used before its declaration.", value),
                            }
                        },
                        "??=" => {
                            let position = self.search_name(value);
                            match position {
                                Some(v) => {
                                    self.byte_operations.borrow_mut().push(OpCode::LoadName(v));
                                    self.byte_operations.borrow_mut().push(OpCode::LoadConst(0));
                                    self.byte_operations.borrow_mut().push(OpCode::compare_op_from_str("=="));
                                    let label_end = self.gen_jump_label();
                                    self.byte_operations.borrow_mut().push(OpCode::PopJumpIfFalse(label_end));
                                    self.compile(right);
                                    self.byte_operations.borrow_mut().push(OpCode::StoreName(v));
                                    self.set_jump_label_value(label_end);
                                },
                                None => panic!("{} is used before its declaration.", value),
                            }
                        },
                        _ => panic!("Unknown assignment operator: {}", value)
                    }
                    // Expressionはスタックになにか残しておきたいので積む
                    self.byte_operations.borrow_mut().push(OpCode::LoadConst(0));
                }
                else {
                    panic!("Invalid AST. Assignment lhs must be an identifier.");
                }
            },
            Node::NumericLiteral { span } => {
                let const_position = self.constant_list.borrow().len() as u8;
                let raw_value = self.span_to_str(span);
                // TODO: 小数と16進数への対応
                if raw_value.starts_with("0x") || raw_value.starts_with("0X") {
                    // 16進数の場合
                    let value = i32::from_str_radix(&raw_value[2..], 16).unwrap();
                    self.constant_list.borrow_mut().push(PyObject::Int(value, false));
                }
                else {
                    match raw_value.parse::<i32>() {
                        Ok(value) => {
                            // 整数の場合
                            self.constant_list.borrow_mut().push(PyObject::Int(value, false));
                        },
                        Err(_) => {
                            // 小数の場合
                            let value = raw_value.parse::<f64>().unwrap();
                            self.constant_list.borrow_mut().push(PyObject::Float(value, false));
                        }
                    }
                }

                self.byte_operations.borrow_mut().push(OpCode::LoadConst(const_position));
            },
            Node::StringLiteral { span } => {
                // let value = self.span_to_str(span);
                let value = &self.source[span.start()..span.end()];
                let len = value.len();
                let value = &value[1..len-1];

                let const_position = self.constant_list.borrow().len() as u8;
                if value.is_ascii() {
                    if value.len() < 256 {
                        self.constant_list.borrow_mut().push(PyObject::AsciiShort(value, false));
                    }
                    else {
                        self.constant_list.borrow_mut().push(PyObject::Ascii(value, false));
                    }
                }
                else {
                    self.constant_list.borrow_mut().push(PyObject::Unicode(value, false));
                }
                self.byte_operations.borrow_mut().push(OpCode::LoadConst(const_position));
            },
            Node::BooleanLiteral { span } => {
                let value = self.span_to_str(span);
                let const_position = self.constant_list.borrow().len() as u8;
                let obj = match value {
                    "true" => PyObject::True(false),
                    "false" => PyObject::False(false),
                    _ => panic!("Unknown Boolean Literal: {}", value)
                };
                self.constant_list.borrow_mut().push(obj);
                self.byte_operations.borrow_mut().push(OpCode::LoadConst(const_position));
            },
            Node::NullLiteral { span: _ } => {
                let const_position = self.constant_list.borrow().len() as u8;
                self.constant_list.borrow_mut().push(PyObject::None(false));
                self.byte_operations.borrow_mut().push(OpCode::LoadConst(const_position));
            },
            Node::Identifier { span } => {
                let value = self.span_to_str(span);
                match self.search_name(value) {
                    Some(p) => {
                        self.byte_operations.borrow_mut().push(OpCode::LoadName(p));
                    },
                    None => {
                        if self.check_variable_defined(value) {
                            let p = self.register_name(value);
                            self.byte_operations.borrow_mut().push(OpCode::LoadName(p));
                        }
                        else {
                            panic!("{} is used before its declaration.", value)
                        }
                    }
                }
            },
            Node::Arguments { span: _, children } => {
                for node in children {
                    self.compile(node);
                }
                self.byte_operations.borrow_mut().push(OpCode::CallFunction(children.len() as u8))
            },
            Node::WithSelectorExpression { span: _, child, selector } => {
                self.compile(child);
                self.compile(selector);
            },

            Node::EmptyStatement { span: _ } => { },
            Node::ExpressionStatement { span: _, expr } => {
                self.compile(expr);
                self.byte_operations.borrow_mut().push(OpCode::PopTop);
            },
            Node::BlockStatement { span: _, children } => {
                for child in children {
                    self.compile(child);
                }
            },

            Node::VariableDeclaration { span: _, identifier, expr } => {
                match expr {
                    Some(e) => {
                        self.compile(e);
                        if let Node::Identifier { span: id_span } = **identifier {
                            let value = self.span_to_str(&id_span);
                            let position = self.register_name(value);
                            self.byte_operations.borrow_mut().push(OpCode::StoreName(position));

                            self.global_variables.borrow_mut().push(value);
                        }
                        else {
                            panic!("Invalid AST");
                        }
                    },
                    None => {
                        if let Node::Identifier { span: id_span } = **identifier {
                            let value = &self.span_to_str(&id_span);
                            self.register_name(value);
                            self.global_variables.borrow_mut().push(value);
                        }
                        else {
                            panic!("Invalid AST");
                        }
                    }
                }
            },
            Node::IfStatement { span: _, condition, if_true_stmt, if_false_stmt } => {
                match if_false_stmt {
                    Some(if_false_stmt) => {
                        // if expr stmt else stmt
                        self.compile(condition);
                        let label_false_starts = self.gen_jump_label();
                        self.byte_operations.borrow_mut().push(OpCode::PopJumpIfFalse(label_false_starts));
                        self.compile(if_true_stmt);
                        
                        let label_if_ends = self.gen_jump_label();
                        self.byte_operations.borrow_mut().push(OpCode::JumpAbsolute(label_if_ends));

                        self.set_jump_label_value(label_false_starts);
                        self.compile(if_false_stmt);

                        self.set_jump_label_value(label_if_ends);
                    },
                    None => {
                        // if expr stmt
                        self.compile(condition);
                        let label_if_ends = self.gen_jump_label();
                        self.byte_operations.borrow_mut().push(OpCode::PopJumpIfFalse(label_if_ends));
                        self.compile(if_true_stmt);

                        self.set_jump_label_value(label_if_ends);
                    }
                }
            },
            Node::ForStatement { span: _, init, condition, update, stmt } => {
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
                    self.byte_operations.borrow_mut().push(OpCode::PopJumpIfFalse(label_for_end));
                }
                self.compile(stmt);
                if let Some(node_list) = update {
                    for node in node_list {
                        self.compile(node);
                        self.byte_operations.borrow_mut().push(OpCode::PopTop);
                    }
                }
                self.byte_operations.borrow_mut().push(OpCode::JumpAbsolute(label_loop_start));
                self.set_jump_label_value(label_for_end);
            },
            Node::WhileStatement { span: _, condition, stmt } => {
                let label_while_end = self.gen_jump_label();
                let label_loop_start = self.gen_jump_label();
                self.set_jump_label_value(label_loop_start);
                self.compile(condition);
                self.byte_operations.borrow_mut().push(OpCode::PopJumpIfFalse(label_while_end));

                self.compile(stmt);
                self.byte_operations.borrow_mut().push(OpCode::JumpAbsolute(label_loop_start));

                self.set_jump_label_value(label_while_end);
            },
            Node::DoStatement { span: _, condition, stmt } => {
                let label_do_start = self.gen_jump_label();
                self.set_jump_label_value(label_do_start);
                self.compile(stmt);

                self.compile(condition);
                self.byte_operations.borrow_mut().push(OpCode::PopJumpIfTrue(label_do_start));
            }
        }
    }

    fn span_to_str(&self, span: &Span) -> &'a str {
        return &self.source[span.start()..span.end()];
    }

    fn register_name(&self, value: &'a str) -> u8 {
        let name_position = self.name_list.borrow().len() as u8;
        if value.is_ascii() {
            if value.len() < 256 {
                self.name_list.borrow_mut().push(PyObject::AsciiShort(value, false));
            }
            else {
                self.name_list.borrow_mut().push(PyObject::Ascii(value, false));
            }
        }
        else {
            self.name_list.borrow_mut().push(PyObject::Unicode(value, false));
        }
        self.name_map.borrow_mut().insert(value, name_position);
        name_position
    
    }

    fn search_name(&self, value: &str) -> Option<u8> {
        self.name_map.borrow().get(value).copied()
    }

    fn check_variable_defined(&self, value: &str) -> bool {
        self.global_variables.borrow().contains(&value)
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

impl <'a>ByteCompiler<'a> {
    pub fn resolve_references(&self) -> Vec<ByteCode> {
        let opcode_list = self.byte_operations.borrow();

        let result = opcode_list.iter().map(|v| {
            v.resolve(&self.jump_label_table.borrow())
        }).collect();

        result
    }
}
