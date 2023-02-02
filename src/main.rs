use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::time::SystemTime;
use std::process::Command;
use std::str;

fn main() {
    println!("Hello, world!");
    let output = "main.pyc";
    {
        let path = Path::new(output);
        match fs::remove_file(path) {
            Result::Ok(_) => println!("file removed"),
            Result::Err(_) => println!("file does not exists"),
        }
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(&path)
            .unwrap();

        write_header(&mut file);
        write_py_code(&mut file);
    }

    match Command::new("bash").args(&["-c","python main.pyc"]).output() {
        Ok(e) => println!("{}", str::from_utf8(&e.stdout).unwrap()),
        Err(e) => println!("Error: {}", e)
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

fn write_py_code(file: &mut File) {
    file.write(&[0xE3]).unwrap(); // ObjectType
    file.write(&(0u32.to_le_bytes())).unwrap(); // ArgCount
    file.write(&(0u32.to_le_bytes())).unwrap(); // PosOnlyArgCount
    file.write(&(0u32.to_le_bytes())).unwrap(); // KwOnlyArgCount
    file.write(&(0u32.to_le_bytes())).unwrap(); // NumLocals
    file.write(&(2u32.to_le_bytes())).unwrap(); // StackSize
    file.write(&(64u32.to_le_bytes())).unwrap(); // Flags

    let codes: [u8; 12] = [0x65, 0x00, 0x64, 0x00, 0x83, 0x01, 0x01, 0x00, 0x64, 0x01, 0x53, 0x00];
    write_py_string(file, &codes);
    {
        // SmallTuple: 定数定義
        let tuple_len = 2u8;
        file.write(&[0x29]).unwrap();
        file.write(&[tuple_len]).unwrap();
        write_py_int(file, 10);
        write_py_none(file);
    }
    {
        // SmallTuple: 名前一覧
        let tuple_len = 1u8;
        file.write(&[0x29]).unwrap();
        file.write(&[tuple_len]).unwrap();
        write_py_short_ascii_interned(file, "print".as_bytes());
    }
    {
        // SmallTuple: ローカル変数一覧
        let tuple_len = 0u8;
        file.write(&[0xA9]).unwrap();
        file.write(&[tuple_len]).unwrap();
    }
    {
        // ObjectRef: 自由変数
        file.write(&[0x72]).unwrap();
        let target = 3u32;
        file.write(&(target.to_le_bytes())).unwrap();
    }
    {
        // ObjectRef: セル変数
        file.write(&[0x72]).unwrap();
        let target = 3u32;
        file.write(&(target.to_le_bytes())).unwrap();
    }
    // ファイル名
    write_py_short_ascii(file, "main.py".as_bytes());
    // 名前
    write_py_short_ascii_interned(file, "<module>".as_bytes());
    // first line
    file.write(&(1u32.to_le_bytes())).unwrap();
    // line table
    {
        file.write(&[0xF3]).unwrap();
        file.write(&(0u32.to_le_bytes())).unwrap();
    }
}

fn write_py_string(file: &mut File, value: &[u8]) {
    file.write(&[0x73]).unwrap(); // ObjectType
    let str_len = value.len() as u32;
    file.write(&(str_len.to_le_bytes())).unwrap();
    file.write(value).unwrap();
}

fn write_py_int(file: &mut File, value: u32) {
    file.write(&[0xE9]).unwrap(); // ObjectType
    file.write(&(value.to_le_bytes())).unwrap();
}

fn write_py_none(file: &mut File) {
    file.write(&[0x4E]).unwrap();
}

fn write_py_short_ascii_interned(file: &mut File, value: &[u8]) {
    file.write(&[0xDA]).unwrap();
    let str_len = value.len() as u8;
    file.write(&[str_len]).unwrap();
    file.write(value).unwrap();
}

fn write_py_short_ascii(file: &mut File, value: &[u8]) {
    file.write(&[0xFA]).unwrap();
    let str_len = value.len() as u8;
    file.write(&[str_len]).unwrap();
    file.write(value).unwrap();
}
