use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::time::SystemTime;

mod bytecode;
mod bytecompiler;
mod parser;
mod pyobject;

use bytecode::{ByteCode, calc_stack_size};

use crate::bytecompiler::ByteCompiler;
use crate::pyobject::PyObject;

pub fn run(output: &str, source: &str) {
    let node = parser::parse(source);
    if node.is_err() {
        println!("{:?}", node.err());
        println!("failed to parse the passed source: {}", source);
        return;
    }
    let node = node.unwrap();
    let compiler = ByteCompiler::run(&node, source);

    let stack_size = calc_stack_size(&compiler.byte_operations.borrow());
    let operation_list = compiler.resolve_references();
    let constant_list = compiler.constant_list.borrow();
    let name_list = compiler.name_list.borrow();

    {
        let path = Path::new(output);
        match fs::remove_file(path) {
            Result::Ok(_) => (), // println!("file removed"),
            Result::Err(_) => println!("file does not exists"),
        }
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(&path)
            .unwrap();

        write_header(&mut file);
        write_py_code(&mut file, stack_size, &operation_list, &constant_list, &name_list);
    }
}

fn write_header(file: &mut File) {
    file.write(&[0x61, 0x0D, 0x0D, 0x0A]).unwrap(); // Magic Number
    file.write(&[0x00, 0x00, 0x00, 0x00]).unwrap(); // Flag(PEP552)

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;
    file.write(&(now.to_le_bytes())).unwrap(); // Timestamp
    let file_size: u32 = 0;
    file.write(&(file_size.to_le_bytes())).unwrap();
}

fn write_py_code(
    file: &mut File,
    stack_size: i32,
    operation_list: &[ByteCode],
    constant_list: &[PyObject],
    name_list: &[PyObject],
) {
    file.write(&[0xE3]).unwrap(); // ObjectType
    file.write(&(0u32.to_le_bytes())).unwrap(); // ArgCount
    file.write(&(0u32.to_le_bytes())).unwrap(); // PosOnlyArgCount
    file.write(&(0u32.to_le_bytes())).unwrap(); // KwOnlyArgCount
    file.write(&(0u32.to_le_bytes())).unwrap(); // NumLocals
    // let stack_size = bytecode::calc_stack_size(&operation_list);
    if stack_size < 0 {
        panic!("invalid stack size: {}", stack_size);
    }
    file.write(&(stack_size as u32).to_le_bytes()).unwrap(); // StackSize
    file.write(&(64u32.to_le_bytes())).unwrap(); // Flags

    let codes = bytecode::compile_code(&operation_list);
    write_py_string(file, &codes, false);

    // 定数一覧
    write_py_small_tuple(file, &constant_list);

    // 名前一覧
    write_py_small_tuple(file, &name_list);

    {
        // SmallTuple: ローカル変数一覧
        let tuple_len = 0u8;
        file.write(&[0xA9]).unwrap();
        file.write(&[tuple_len]).unwrap();
    }
    {
        // ObjectRef: 自由変数
        // file.write(&[0x72]).unwrap();
        // let target = 3u32;
        // file.write(&(target.to_le_bytes())).unwrap();
        let tuple_len = 0u8;
        file.write(&[0xA9]).unwrap();
        file.write(&[tuple_len]).unwrap();
    }
    {
        // ObjectRef: セル変数
        // file.write(&[0x72]).unwrap();
        // let target = 3u32;
        // file.write(&(target.to_le_bytes())).unwrap();
        let tuple_len = 0u8;
        file.write(&[0xA9]).unwrap();
        file.write(&[tuple_len]).unwrap();
    }
    // ファイル名
    write_py_short_ascii(file, "main.py".as_bytes(), true);
    // 名前
    write_py_short_ascii(file, "<module>".as_bytes(), true);
    // first line
    file.write(&(1u32.to_le_bytes())).unwrap();
    // line table
    {
        file.write(&[0xF3]).unwrap();
        file.write(&(0u32.to_le_bytes())).unwrap();
    }
}

fn write_py_string(file: &mut File, value: &[u8], register_ref: bool) {
    let object_type = 0x73u8 | ((register_ref as u8) << 7);
    file.write(&[object_type]).unwrap(); // ObjectType
    let str_len = value.len() as u32;
    file.write(&(str_len.to_le_bytes())).unwrap();
    file.write(value).unwrap();
}

fn write_py_int(file: &mut File, value: i32, register_ref: bool) {
    let object_type = 0x69u8 | ((register_ref as u8) << 7);
    file.write(&[object_type]).unwrap(); // ObjectType
    file.write(&(value.to_le_bytes())).unwrap();
}

fn write_py_float64(file: &mut File, value: f64, register_ref: bool) {
    let object_type = 0x67u8 | ((register_ref as u8) << 7);
    file.write(&[object_type]).unwrap(); // ObjectType
    file.write(&(value.to_le_bytes())).unwrap();
}

fn write_py_none(file: &mut File, register_ref: bool) {
    let object_type = 0x4Eu8 | ((register_ref as u8) << 7);
    file.write(&[object_type]).unwrap();
}

// fn write_py_short_ascii_interned(file: &mut File, value: &[u8], register_ref: bool) {
//     let object_type = 0x5Au8 | ((register_ref as u8) << 7);
//     file.write(&[object_type]).unwrap();
//     let str_len = value.len() as u8;
//     file.write(&[str_len]).unwrap();
//     file.write(value).unwrap();
// }

fn write_py_short_ascii(file: &mut File, value: &[u8], register_ref: bool) {
    let object_type = 0x7Au8 | ((register_ref as u8) << 7);
    file.write(&[object_type]).unwrap();
    let str_len = value.len() as u8;
    file.write(&[str_len]).unwrap();
    file.write(value).unwrap();
}

fn write_py_ascii(file: &mut File, value: &[u8], register_ref: bool) {
    let object_type = 0x61u8 | ((register_ref as u8) << 7);
    file.write(&[object_type]).unwrap();
    let str_len = value.len() as u32;
    file.write(&str_len.to_le_bytes()).unwrap();
    file.write(value).unwrap();
}

fn write_py_unicode(file: &mut File, value: &[u8], register_ref: bool) {
    let object_type = 0x75u8 | ((register_ref as u8) << 7);
    file.write(&[object_type]).unwrap();
    let str_len = value.len() as u32;
    file.write(&str_len.to_le_bytes()).unwrap();
    file.write(value).unwrap();
}

fn write_py_boolean(file: &mut File, value: bool, register_ref: bool) {
    if value {
        let object_type = 0x54u8 | ((register_ref as u8) << 7);
        file.write(&[object_type]).unwrap();
    }
    else {
        let object_type = 0x46u8 | ((register_ref as u8) << 7);
        file.write(&[object_type]).unwrap();
    }
}

fn write_py_small_tuple(file: &mut File, value_list: &[PyObject]) {
    let tuple_len = value_list.len() as u8;
    file.write(&[0x29]).unwrap();
    file.write(&[tuple_len]).unwrap();
    for c in value_list {
        match *c {
            PyObject::Int(v, r) => write_py_int(file, v, r),
            PyObject::Float(v, r) => write_py_float64(file, v, r),
            PyObject::String(v, r) => write_py_string(file, v.as_bytes(), r),
            PyObject::Ascii(v, r) => write_py_ascii(file, v.as_bytes(), r),
            PyObject::AsciiShort(v, r) => write_py_short_ascii(file, v.as_bytes(), r),
            PyObject::Unicode(v, r) => write_py_unicode(file, v.as_bytes(), r),
            PyObject::None(r) => write_py_none(file, r),
            PyObject::True(r) => write_py_boolean(file, true, r),
            PyObject::False(r) => write_py_boolean(file, false, r)
        }
    }
}