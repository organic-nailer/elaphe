use std::{rc::Rc, cell::RefCell, collections::HashMap};

use crate::bytecode::{OpCode, calc_stack_size};
use crate::executioncontext::{PyContext, BlockContext};
use crate::pyobject::PyObject;
use crate::parser::{Node};

use super::ByteCompiler;

pub fn run_class<'ctx, 'value, 'cpl>(
    file_name: &'value str,
    code_name: &'value str,
    member_list: &'value Vec<Box<Node>>,
    outer_compiler: &'cpl ByteCompiler<'ctx, 'value>,
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

    // __module__ = __name__
    let p = (compiler.context_stack.last().unwrap())
        .borrow_mut()
        .register_or_get_name("__name__".to_string());
    compiler.push_op(OpCode::LoadName(p));
    
    let p = (compiler.context_stack.last().unwrap())
        .borrow_mut()
        .register_or_get_name("__module__".to_string());
    compiler.push_op(OpCode::StoreName(p));

    // __qualname__ = "Hoge"
    compiler.push_load_const(PyObject::new_string("Hoge".to_string(), false));
    let p = (compiler.context_stack.last().unwrap())
        .borrow_mut()
        .register_or_get_name("__qualname__".to_string());
    compiler.push_op(OpCode::StoreName(p));


    for member in member_list {
        compile_member(&mut compiler, member, code_name);
    }


    // 終わり
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
        num_args: 0,
        num_pos_only_args: 0,
        num_kw_only_args: 0,
        num_locals: 0,
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
            children: vec![],
            add_ref: false,
        }),
        add_ref: false,
    }
}

fn compile_member<'ctx, 'value, 'cpl>(compiler: &'cpl mut ByteCompiler<'ctx, 'value>, node: &'value Box<Node>, class_name: &'value str) {
    match &**node {
        Node::FunctionDeclaration { span:_, signature, body } => {
            let prefix = format!("{}{}", class_name, ".");
            compiler.comiple_declare_function(&signature, &body, Some(prefix), Some("self"));
        },
        _ => panic!("Not Implemented Member")
    }
}