use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::bytecode::{calc_stack_size, OpCode};
use crate::executioncontext::{BlockContext, ExecutionContext, PyContext};
use crate::parser::node::NodeStatement;
use crate::pyobject::PyObject;

use super::ByteCompiler;

pub fn run_function<'ctx, 'value, 'cpl, F: FnOnce(&mut ByteCompiler<'ctx, 'value>)>(
    file_name: &String,
    code_name: &String,
    argument_list: Vec<String>,
    num_args: u32,
    num_pos_only_args: u32,
    num_kw_only_args: u32,
    outer_compiler: &'cpl ByteCompiler<'ctx, 'value>,
    body: &'value NodeStatement,
    source: &'value str,
    preface: F,
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

    for arg in argument_list {
        (*block_context).borrow_mut().declare_variable(&arg);
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

    preface(&mut compiler);

    compiler.compile_stmt(body, None);

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
        num_pos_only_args,
        num_kw_only_args,
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
