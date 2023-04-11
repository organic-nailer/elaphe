use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::time::SystemTime;

use dart_parser_generator::parser_generator;
use ciborium::de;
use parser::node::NodeExpression;

mod bytecode;
mod bytecompiler;
mod executioncontext;
mod parser;
mod pyobject;
mod tokenizer;

pub fn run(output: &str, source: &str) -> Result<(), ()> {
    // Tokenize
    let token_list = tokenizer::tokenize(source);

    // Parse
    let reader = std::fs::File::open(concat!(env!("OUT_DIR"), "/parser.bin")).unwrap();
    let transition_map: parser_generator::TransitionMap = de::from_reader(reader).unwrap();
    let node = parser::parse(token_list, transition_map);
    if node.is_err() {
        println!("{:?}", node.err());
        println!("failed to parse the passed source: {}", source);
        return Err(());
    }
    let node_list = node.unwrap();

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
        write_root_py_code(&mut file, source, node_list);
    }
    Ok(())
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

fn write_root_py_code(file: &mut File, source: &str, node: NodeExpression) {
    let code = bytecompiler::runroot::run_root(&"main.py".to_string(), &node, source);

    code.write(file);
}
