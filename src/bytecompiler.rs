use std::cell::RefCell;
use std::i32;

use cfgrammar::Span;

use crate::{bytecode::OpCode, parser::Node, pyobject::PyObject};

pub struct ByteCompiler<'a> {
    pub byte_operations: RefCell<Vec<OpCode>>,
    pub constant_list: RefCell<Vec<PyObject<'a>>>,
    pub name_list: RefCell<Vec<PyObject<'a>>>,
    source: &'a str,
}

impl ByteCompiler<'_> {
    pub fn run<'a>(root_node: &Node, source: &'a str) -> ByteCompiler<'a> {
        let compiler = ByteCompiler {
            byte_operations: RefCell::new(vec![]),
            constant_list: RefCell::new(vec![]),
            name_list: RefCell::new(vec![]),
            source: source,
        };

        compiler.constant_list.borrow_mut().push(PyObject::None(false));

        compiler.compile(root_node);

        compiler.byte_operations.borrow_mut().push(OpCode::PopTop);
        compiler
            .byte_operations
            .borrow_mut()
            .push(OpCode::LoadConst(0));
        compiler.byte_operations.borrow_mut().push(OpCode::ReturnValue);

        compiler
    }
}

impl ByteCompiler<'_> {
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
            }
            Node::UnaryOpExpression { span: _, operator, child } => {
                self.compile(child);
                match *operator {
                    "-" => self.byte_operations.borrow_mut().push(OpCode::UnaryNegative),
                    "!" => self.byte_operations.borrow_mut().push(OpCode::UnaryNot),
                    "~" => self.byte_operations.borrow_mut().push(OpCode::UnaryInvert),
                    _ => panic!("unknown unary operator: {}", *operator),
                }
            }
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
            }
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
                let value = &self.source[span.start()..span.end()];
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
                self.byte_operations.borrow_mut().push(OpCode::LoadName(name_position));
            }
            Node::Arguments { span: _, children } => {
                for node in children {
                    self.compile(node);
                }
                self.byte_operations.borrow_mut().push(OpCode::CallFunction(children.len() as u8))
            },
            Node::WithSelectorExpression { span: _, child, selector } => {
                self.compile(child);
                self.compile(selector);
            }
        }
    }

    fn span_to_str<'a>(&'a self, span: &Span) -> &'a str {
        return &self.source[span.start()..span.end()];
    }
}
