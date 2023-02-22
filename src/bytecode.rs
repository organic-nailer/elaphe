use std::{cmp, collections::HashMap};

pub struct ByteCode {
    operation: u8,
    operand: u8,
}

pub enum OpCode {
    PopTop,
    RotTwo,
    DupTop,
    UnaryNegative,
    UnaryNot,
    UnaryInvert,
    BinaryMultiply,
    BinaryModulo,
    BinaryAdd,
    BinarySubtract,
    BinaryFloorDivide,
    BinaryTrueDivide,
    InplaceFloorDivide,
    InplaceTrueDivide,
    Reraise,
    InplaceAdd,
    InplaceSubtract,
    InplaceMultiply,
    InplaceModulo,
    BinaryLShift,
    BinaryRShift,
    BinaryAnd,
    BinaryXor,
    BinaryOr,
    InplaceLShift,
    InplaceRShift,
    InplaceAnd,
    InplaceXor,
    InplaceOr,
    ReturnValue,
    PopBlock,
    PopExcept,
    StoreName(u8),
    StoreGlobal(u8),
    LoadConst(u8),
    LoadName(u8),
    LoadAttr(u8),
    CompareOp(u8),
    ImportName(u8),
    ImportFrom(u8),
    // JumpForward(u32),
    // JumpIfFalseOrPop(u32),
    // JumpIfTrueOrPop(u32),
    JumpAbsolute(u32),
    PopJumpIfFalse(u32),
    LoadGlobal(u8),
    JumpIfNotExcMatch(u32),
    SetupFinally(u32),
    LoadFast(u8),
    StoreFast(u8),
    RaiseVarargs(u8),
    PopJumpIfTrue(u32),
    CallFunction(u8),
    MakeFunction, // フラグを持つらしい
    LoadMethod(u8),
    CallMethod(u8),
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
            _ => panic!("Unknown compare op: {}", op),
        };
        OpCode::CompareOp(operand)
    }
}

impl OpCode {
    fn get_value(&self) -> u8 {
        match *self {
            OpCode::PopTop => 1,
            OpCode::RotTwo => 2,
            OpCode::DupTop => 4,
            OpCode::UnaryNegative => 11,
            OpCode::UnaryNot => 12,
            OpCode::UnaryInvert => 15,
            OpCode::BinaryMultiply => 20,
            OpCode::BinaryModulo => 22,
            OpCode::BinaryAdd => 23,
            OpCode::BinarySubtract => 24,
            OpCode::BinaryFloorDivide => 26,
            OpCode::BinaryTrueDivide => 27,
            OpCode::InplaceFloorDivide => 28,
            OpCode::InplaceTrueDivide => 29,
            OpCode::Reraise => 48,
            OpCode::InplaceAdd => 55,
            OpCode::InplaceSubtract => 56,
            OpCode::InplaceMultiply => 57,
            OpCode::InplaceModulo => 59,
            OpCode::BinaryLShift => 62,
            OpCode::BinaryRShift => 63,
            OpCode::BinaryAnd => 64,
            OpCode::BinaryXor => 65,
            OpCode::BinaryOr => 66,
            OpCode::InplaceLShift => 75,
            OpCode::InplaceRShift => 76,
            OpCode::InplaceAnd => 77,
            OpCode::InplaceXor => 78,
            OpCode::InplaceOr => 79,
            OpCode::ReturnValue => 83,
            OpCode::PopBlock => 87,
            OpCode::PopExcept => 89,
            OpCode::StoreName(_) => 90,
            OpCode::StoreGlobal(_) => 97,
            OpCode::LoadConst(_) => 100,
            OpCode::LoadName(_) => 101,
            OpCode::LoadAttr(_) => 106,
            OpCode::CompareOp(_) => 107,
            OpCode::ImportName(_) => 108,
            OpCode::ImportFrom(_) => 109,
            // OpCode::JumpForward(_) => 110,
            // OpCode::JumpIfFalseOrPop(_) => 111,
            // OpCode::JumpIfTrueOrPop(_) => 112,
            OpCode::JumpAbsolute(_) => 113,
            OpCode::PopJumpIfFalse(_) => 114,
            OpCode::PopJumpIfTrue(_) => 115,
            OpCode::LoadGlobal(_) => 116,
            OpCode::JumpIfNotExcMatch(_) => 121,
            OpCode::SetupFinally(_) => 122,
            OpCode::LoadFast(_) => 124,
            OpCode::StoreFast(_) => 125,
            OpCode::RaiseVarargs(_) => 130,
            OpCode::CallFunction(_) => 131,
            OpCode::MakeFunction => 132,
            OpCode::LoadMethod(_) => 160,
            OpCode::CallMethod(_) => 161,
        }
    }

    pub fn resolve(&self, label_table: &HashMap<u32, u8>) -> ByteCode {
        match *self {
            // OpCode::JumpForward(v) |
            // OpCode::JumpIfFalseOrPop(v) |
            // OpCode::JumpIfTrueOrPop(v) |
            OpCode::JumpAbsolute(v) 
            | OpCode::PopJumpIfFalse(v) 
            | OpCode::PopJumpIfTrue(v) 
            | OpCode::JumpIfNotExcMatch(v)
            | OpCode::SetupFinally(v) => {
                let operand = *label_table.get(&v).unwrap();
                ByteCode {
                    operation: self.get_value(),
                    operand: operand,
                }
            }
            OpCode::StoreName(v)
            | OpCode::LoadConst(v)
            | OpCode::LoadName(v)
            | OpCode::LoadGlobal(v)
            | OpCode::StoreGlobal(v)
            | OpCode::LoadFast(v)
            | OpCode::StoreFast(v)
            | OpCode::CallFunction(v)
            | OpCode::LoadAttr(v)
            | OpCode::CompareOp(v)
            | OpCode::ImportName(v)
            | OpCode::ImportFrom(v)
            | OpCode::LoadMethod(v)
            | OpCode::CallMethod(v)
            | OpCode::RaiseVarargs(v) => ByteCode {
                operation: self.get_value(),
                operand: v,
            },
            _ => ByteCode {
                operation: self.get_value(),
                operand: 0,
            },
        }
    }

    // https://github.com/python/cpython/blob/b2b85b5db9cfdb24f966b61757536a898abc3830/Python/compile.c#L1075
    pub fn stack_effect(&self, jump: bool) -> i32 {
        match *self {
            OpCode::PopTop => -1,

            OpCode::DupTop => 1,

            OpCode::UnaryNegative | OpCode::UnaryNot | OpCode::UnaryInvert | OpCode::RotTwo => 0,

            OpCode::BinaryAdd
            | OpCode::BinaryMultiply
            | OpCode::BinaryModulo
            | OpCode::BinarySubtract
            | OpCode::BinaryTrueDivide
            | OpCode::BinaryFloorDivide
            | OpCode::BinaryLShift
            | OpCode::BinaryRShift
            | OpCode::BinaryAnd
            | OpCode::BinaryXor
            | OpCode::BinaryOr
            | OpCode::InplaceFloorDivide
            | OpCode::InplaceTrueDivide
            | OpCode::InplaceAdd
            | OpCode::InplaceSubtract
            | OpCode::InplaceMultiply
            | OpCode::InplaceModulo
            | OpCode::InplaceLShift
            | OpCode::InplaceRShift
            | OpCode::InplaceAnd
            | OpCode::InplaceXor
            | OpCode::InplaceOr
            | OpCode::CompareOp(_) => -1,

            OpCode::ImportName(_) => -1,
            OpCode::ImportFrom(_) => 1,

            OpCode::StoreGlobal(_) | OpCode::StoreFast(_) | OpCode::StoreName(_) => -1,

            OpCode::ReturnValue => -1,
            OpCode::PopBlock => 0,
            OpCode::PopExcept => -1,

            OpCode::LoadConst(_)
            | OpCode::LoadName(_)
            | OpCode::LoadFast(_)
            | OpCode::LoadGlobal(_) => 1,

            OpCode::CallMethod(n) | OpCode::CallFunction(n) => -(n as i32),

            OpCode::SetupFinally(_) => if jump { 1 } else { 0 },

            OpCode::Reraise => -1,
            OpCode::RaiseVarargs(v) => -(v as i32),

            // OpCode::JumpForward(_) |
            OpCode::JumpAbsolute(_) => 0,
            // OpCode::JumpIfFalseOrPop(_) |
            // OpCode::JumpIfTrueOrPop(_) => if jump { 0 } else { -1 },
            OpCode::PopJumpIfFalse(_) | OpCode::PopJumpIfTrue(_) => -1,
            OpCode::JumpIfNotExcMatch(_) => 0,

            OpCode::MakeFunction => -1,

            OpCode::LoadAttr(_) | OpCode::LoadMethod(_) => 0,
        }
    }
}

pub fn compile_code(operation_list: &[ByteCode]) -> Vec<u8> {
    let code_size = operation_list.len() * 2;
    let mut result = vec![0u8; code_size];
    let mut i = 0;
    for op in operation_list {
        result[i] = op.operation;
        result[i + 1] = op.operand;
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
