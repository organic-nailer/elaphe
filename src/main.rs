use anyhow::{anyhow, Result};
use elaphe::{build_from_code, build_from_file};
use getopts::Options;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;
use std::{env, fs};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let now = std::time::SystemTime::now();

    if args.len() == 1 {
        panic!("invalid arguments");
    }
    let command = &args[1];
    if command == "run" {
        let mut opts = Options::new();
        opts.optopt("c", "", "eval string", "CODE");
        let matches = match opts.parse(&args[2..]) {
            Ok(m) => m,
            Err(f) => {
                panic!("{}", f.to_string())
            }
        };

        if !matches.free.is_empty() {
            // ファイル名で実行
            let file_name = matches.free[0].clone();
            let output = Path::new(&file_name).with_extension("pyc");
            let output = output.to_str().unwrap();

            build_from_file(output, &file_name, now, true)?;
            execute_pyc(output)
        } else {
            // 文字列を実行
            let source = matches.opt_str("c");

            let output = "main.pyc";
            match source {
                Some(source) => {
                    build_from_code(output, &source, now)?;
                    execute_pyc(output)
                }
                None => Err(anyhow!("invalid arguments")),
            }
        }
    } else if command == "build" {
        let file_name = args[2].clone();
        let output = Path::new(&file_name).with_extension("pyc");
        let output = output.to_str().unwrap();
        build_from_file(output, &file_name, now, true)
    } else if command == "init" {
        let dir = &args[2];
        elaphe_init(dir)?;
        Ok(())
    } else if command == "add" {
        let package_name = &args[2];
        elaphe_add(package_name)?;
        Ok(())
    } else {
        Err(anyhow!("invalid arguments"))
    }
}

// fn compile_only(output: &str, source: &str) -> Result<(), Box<dyn Error>> {
//     match elaphe::run(output, &source) {
//         Ok(_) => println!("{} is generated!", output),
//         Err(_) => {
//             return Err("".into());
//         }
//     }
//     Ok(())
// }

fn execute_pyc(file_name: &str) -> Result<()> {
    println!("run {}", file_name);
    match Command::new("python").args(&[file_name]).output() {
        Ok(e) => {
            println!("----- result -----");
            println!("{}", str::from_utf8(&e.stdout).unwrap());
            println!("------ end -------");
        }
        Err(e) => println!("Error: {}", e),
    }
    Ok(())
}

fn elaphe_init(dir: &str) -> Result<()> {
    let mut path = env::current_dir()?;
    path.push(dir);
    if !path.exists() {
        fs::create_dir(&path)?;
    }
    let exec_path = env::current_exe()?;
    let exec_dir = exec_path.parent().unwrap();
    copy_directory_contents(&exec_dir.join("template"), &path)?;
    Ok(())
}

fn elaphe_add(package_name: &str) -> Result<()> {
    println!("add {}", package_name);

    match Command::new("python")
        .args(&["-u", "script/gen_type_stubs.py", package_name])
        .output()
    {
        Ok(e) => {
            println!("{}", str::from_utf8(&e.stdout).unwrap());
        }
        Err(e) => println!("Error: {}", e),
    }

    Ok(())
}

fn copy_directory_contents(source: &Path, destination: &Path) -> Result<()> {
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let path = entry.path();
        let new_file_name = path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .trim_start_matches("_")
            .to_string();
        let new_path = PathBuf::from(destination).join(new_file_name);

        if path.is_dir() {
            fs::create_dir(&new_path)?;
            copy_directory_contents(&path, &new_path)?;
        } else {
            fs::copy(&path, &new_path)?;
        }
    }
    Ok(())
}
