use cfgrammar::Span;

use crate::{bytecode::OpCode, parser::Node, pyobject::PyObject};

pub struct ByteCompiler<'a> {
    pub byte_operations: Vec<OpCode>,
    pub constant_list: Vec<PyObject>,
    pub name_list: Vec<PyObject>,
    source: &'a str,
}

impl ByteCompiler<'_> {
    pub fn run<'a>(root_node: &Node, source: &'a str) -> ByteCompiler<'a> {
        let mut compiler = ByteCompiler {
            byte_operations: vec![],
            constant_list: vec![],
            name_list: vec![],
            source: source,
        };
        compiler.compile(root_node);

        // print(x); return None;
        let print_position = compiler.name_list.len() as u8;
        compiler.name_list.push(PyObject::Ascii("print", true));
        let none_position = compiler.constant_list.len() as u8;
        compiler.constant_list.push(PyObject::None(false));
        compiler
            .byte_operations
            .insert(0, OpCode::LoadName(print_position));
        compiler.byte_operations.push(OpCode::CallFunction(1));
        compiler.byte_operations.push(OpCode::PopTop);
        compiler
            .byte_operations
            .push(OpCode::LoadConst(none_position));
        compiler.byte_operations.push(OpCode::ReturnValue);

        compiler
    }
}

impl ByteCompiler<'_> {
    fn compile(&mut self, node: &Node) {
        match node {
            Node::BinaryExpression {
                span,
                operator,
                left,
                right,
            } => {
                self.compile(left);
                self.compile(right);
                match *operator {
                    "+" => self.byte_operations.push(OpCode::BinaryAdd),
                    "-" => self.byte_operations.push(OpCode::BinarySubtract),
                    "*" => self.byte_operations.push(OpCode::BinaryMultiply),
                    "/" => self.byte_operations.push(OpCode::BinaryTrueDivide),
                    _ => panic!("unknown operator: {}", *operator),
                }
            }
            Node::NumericLiteral { span } => {
                let raw_value = self.span_to_str(span);
                let value: i32 = raw_value.parse().unwrap();

                let const_position = self.constant_list.len() as u8;
                self.constant_list.push(PyObject::Int(value, false));
                self.byte_operations.push(OpCode::LoadConst(const_position));
            }
        }
    }

    fn span_to_str(&self, span: &Span) -> &str {
        return &self.source[span.start()..span.end()];
    }
}
