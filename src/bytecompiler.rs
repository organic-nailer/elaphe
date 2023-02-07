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
        compiler.compile(root_node);

        // print(x); return None;
        let print_position = compiler.name_list.borrow().len() as u8;
        compiler.name_list.borrow_mut().push(PyObject::Ascii("print", true));
        let none_position = compiler.constant_list.borrow().len() as u8;
        compiler.constant_list.borrow_mut().push(PyObject::None(false));
        compiler
            .byte_operations
            .borrow_mut()
            .insert(0, OpCode::LoadName(print_position));
        compiler.byte_operations.borrow_mut().push(OpCode::CallFunction(1));
        compiler.byte_operations.borrow_mut().push(OpCode::PopTop);
        compiler
            .byte_operations
            .borrow_mut()
            .push(OpCode::LoadConst(none_position));
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
                self.constant_list.borrow_mut().push(PyObject::Str(value, false));
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
            }
        }
    }

    fn span_to_str<'a>(&'a self, span: &Span) -> &'a str {
        return &self.source[span.start()..span.end()];
    }
}
