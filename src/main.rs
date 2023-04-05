use getopts::Options;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;
use std::{env, fs};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

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

        let output = "main.pyc";

        if !matches.free.is_empty() {
            // ファイル名で実行
            let file_name = matches.free[0].clone();
            let source = fs::read_to_string(file_name)?;

            compile_and_run(output, &source)?;
            Ok(())
        } else {
            // 文字列を実行
            let source = matches.opt_str("c");
            match source {
                Some(source) => {
                    compile_and_run(output, &source)?;
                    Ok(())
                }
                None => Err("invalid arguments".into()),
            }
        }
    } else if command == "build" {
        let output = "main.pyc";
        let file_name = args[2].clone();
        let source = fs::read_to_string(file_name)?;
        compile_only(output, &source)?;
        Ok(())
    } else if command == "init" {
        let dir = &args[2];
        elaphe_init(dir)?;
        Ok(())
    } else if command == "add" {
        let package_name = &args[2];
        elaphe_add(package_name)?;
        Ok(())
    } else {
        Err("invalid command".into())
    }
}

fn compile_only(output: &str, source: &str) -> Result<(), Box<dyn Error>> {
    match elaphe::run(output, &source) {
        Ok(_) => println!("{} is generated!", output),
        Err(_) => {
            return Err("".into());
        }
    }
    Ok(())
}

fn compile_and_run(output: &str, source: &str) -> Result<(), Box<dyn Error>> {
    match elaphe::run(output, &source) {
        Ok(_) => println!("{} is generated!", output),
        Err(_) => {
            return Err("".into());
        }
    }

    println!("run {}", output);
    match Command::new("python")
        .args(&["main.pyc"])
        .output()
    {
        Ok(e) => {
            println!("----- result -----");
            println!("{}", str::from_utf8(&e.stdout).unwrap());
            println!("------ end -------");
        }
        Err(e) => println!("Error: {}", e),
    }
    Ok(())
}

fn elaphe_init(dir: &str) -> Result<(), Box<dyn Error>> {
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

fn elaphe_add(package_name: &str) -> Result<(), Box<dyn Error>> {
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

fn copy_directory_contents(source: &Path, destination: &Path) -> Result<(), Box<dyn Error>> {
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
