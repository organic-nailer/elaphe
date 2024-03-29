use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::time::SystemTime;

use anyhow::{Context, Result};
use ciborium::de;
use dart_parser_generator::parser_generator;
use parser::node::LibraryDeclaration;

mod bytecode;
mod bytecompiler;
mod executioncontext;
mod parser;
mod pyobject;
mod tokenizer;

pub fn build_from_file(
    output: &str,
    source_file: &str,
    time_start_build: SystemTime,
    is_root: bool,
) -> Result<()> {
    let source = fs::read_to_string(source_file).unwrap();
    run(output, &source, time_start_build, is_root)
}

pub fn build_from_code(output: &str, code: &str, time_start_build: SystemTime) -> Result<()> {
    run(output, &code, time_start_build, true)
}

pub fn build_from_code_single(output: &str, code: &str) -> Result<()> {
    run(output, &code, SystemTime::now(), true)
}

fn run(output: &str, source: &str, time_start_build: SystemTime, is_root: bool) -> Result<()> {
    // Tokenize
    let token_list = tokenizer::tokenize(source)
        .with_context(|| format!("failed to tokenize the passed source: {}", source))?;

    // Parse
    let reader = std::fs::File::open(concat!(env!("OUT_DIR"), "/parser.bin")).unwrap();
    let transition_map: parser_generator::TransitionMap = de::from_reader(reader).unwrap();
    let node = parser::parse(token_list, transition_map)
        .with_context(|| format!("failed to parse the passed source: {}", source))?;

    {
        let path = Path::new(output);
        match fs::remove_file(path) {
            Result::Ok(_) => (),  // println!("file removed"),
            Result::Err(_) => (), // println!("file does not exists"),
        }
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(&path)
            .unwrap();
        let filename = path
            .with_extension("pyc")
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        write_header(&mut file);
        write_root_py_code(&mut file, filename, source, node, time_start_build, is_root)?;
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

fn write_root_py_code(
    file: &mut File,
    file_name: String,
    source: &str,
    node: LibraryDeclaration,
    time_start_build: SystemTime,
    is_root: bool,
) -> Result<()> {
    let code =
        bytecompiler::runroot::run_root(&file_name, &node, source, time_start_build, is_root)?;

    code.write(file)?;
    Ok(())
}
