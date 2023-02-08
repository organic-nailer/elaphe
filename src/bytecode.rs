use std::{cmp, collections::HashMap};

pub struct ByteCode {
    operation: u8,
    operand: u8,
}

pub enum OpCode {
    PopTop,
    BinaryAdd,
    BinarySubtract,
    BinaryMultiply,
    BinaryTrueDivide,
    BinaryLShift,
    BinaryRShift,
    BinaryAnd,
    BinaryOr,
    BinaryXor,
    UnaryNegative,
    UnaryNot,
    UnaryInvert,
    ReturnValue,
    CompareOp(u8),
    // JumpForward(u32),
    // JumpIfFalseOrPop(u32),
    // JumpIfTrueOrPop(u32),
    JumpAbsolute(u32),
    PopJumpIfFalse(u32),
    PopJumpIfTrue(u32),
    LoadConst(u8),
    LoadName(u8),
    StoreName(u8),
    CallFunction(u8),
}

impl OpCode {
    pub fn compare_op_from_str(op: &str) -> OpCode {
        let operand: u8 = match op {
            "<" => 0,
            "<=" => 1,
            "==" => 2,
            "!=" => 3,
            ">" => 4,
            ">=" => 5,
            _ => panic!("Unknown compare op: {}", op)
        };
        OpCode::CompareOp(operand)
    }
}

impl OpCode {
    fn get_value(&self) -> u8 {
        match *self {
            OpCode::PopTop => 1,
            OpCode::UnaryNegative => 11,
            OpCode::UnaryNot => 12,
            OpCode::UnaryInvert => 15,
            OpCode::BinaryMultiply => 20,
            OpCode::BinaryAdd => 23,
            OpCode::BinarySubtract => 24,
            OpCode::BinaryTrueDivide => 27,
            OpCode::BinaryLShift => 62,
            OpCode::BinaryRShift => 63,
            OpCode::BinaryAnd => 64,
            OpCode::BinaryXor => 65,
            OpCode::BinaryOr => 66,
            OpCode::ReturnValue => 83,
            OpCode::StoreName(_) => 90,
            OpCode::LoadConst(_) => 100,
            OpCode::LoadName(_) => 101,
            OpCode::CompareOp(_) => 107,
            // OpCode::JumpForward(_) => 110,
            // OpCode::JumpIfFalseOrPop(_) => 111,
            // OpCode::JumpIfTrueOrPop(_) => 112,
            OpCode::JumpAbsolute(_) => 113,
            OpCode::PopJumpIfFalse(_) => 114,
            OpCode::PopJumpIfTrue(_) => 115,
            OpCode::CallFunction(_) => 131
        }
    }

    pub fn resolve(&self, label_table: &HashMap<u32,u8>) -> ByteCode {
        match *self {
            // OpCode::JumpForward(v) |
            // OpCode::JumpIfFalseOrPop(v) |
            // OpCode::JumpIfTrueOrPop(v) |
            OpCode::JumpAbsolute(v) |
            OpCode::PopJumpIfFalse(v) |
            OpCode::PopJumpIfTrue(v) => {
                let operand = *label_table.get(&v).unwrap();
                ByteCode {
                    operation: self.get_value(),
                    operand: operand
                }
            },
            OpCode::StoreName(v) |
            OpCode::LoadConst(v) |
            OpCode::LoadName(v) |
            OpCode::CallFunction(v) |
            OpCode::CompareOp(v) => {
                ByteCode {
                    operation: self.get_value(),
                    operand: v
                }
            },
            _ => {
                ByteCode {
                    operation: self.get_value(),
                    operand: 0
                }
            }
        }
    }

    // https://github.com/python/cpython/blob/b2b85b5db9cfdb24f966b61757536a898abc3830/Python/compile.c#L1075
    pub fn stack_effect(&self, jump: bool) -> i32 {
        match *self {
            OpCode::PopTop => -1,
            
            OpCode::UnaryNegative |
            OpCode::UnaryNot |
            OpCode::UnaryInvert => 0,

            OpCode::BinaryAdd |
            OpCode::BinaryMultiply |
            OpCode::BinarySubtract |
            OpCode::BinaryTrueDivide |
            OpCode::BinaryLShift |
            OpCode::BinaryRShift |
            OpCode::BinaryAnd |
            OpCode::BinaryXor |
            OpCode::BinaryOr |
            OpCode::CompareOp(_) => -1,

            OpCode::StoreName(_) => -1,

            OpCode::ReturnValue => -1,

            OpCode::LoadConst(_) |
            OpCode::LoadName(_) => 1,
            
            OpCode::CallFunction(n) => -(n as i32),

            // OpCode::JumpForward(_) |
            OpCode::JumpAbsolute(_) => 0,
            // OpCode::JumpIfFalseOrPop(_) |
            // OpCode::JumpIfTrueOrPop(_) => if jump { 0 } else { -1 },
            OpCode::PopJumpIfFalse(_) |
            OpCode::PopJumpIfTrue(_) => -1
        }
    }
}

pub fn compile_code(operation_list: &[ByteCode]) -> Vec<u8> {
    let code_size = operation_list.len() * 2;
    let mut result = vec![0u8; code_size];
    let mut i = 0;
    for op in operation_list {
        result[i] = op.operation;
        result[i+1] = op.operand;
        i += 2;
    }
    result
}

pub fn calc_stack_size(operation_list: &[OpCode]) -> i32 {
    let mut max_size = 0;
    let mut current_size = 0;
    for op in operation_list {
        current_size += op.stack_effect(true);
        max_size = cmp::max(max_size, current_size);
    }
    max_size
}