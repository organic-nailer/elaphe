use anyhow::{anyhow, ensure, Context, Result};
use elaphe::{build_from_code, build_from_file};
use getopts::Options;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;
use std::{env, fs};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let now = std::time::SystemTime::now();

    ensure!(args.len() >= 2, "invalid arguments. please input command.");

    let command = &args[1];
    if command == "run" {
        let mut opts = Options::new();
        opts.optopt("c", "", "eval string", "CODE");
        let matches = opts
            .parse(&args[2..])
            .with_context(|| "failed to parse arguments")?;

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
    let output = Command::new("python")
        .args(&[file_name])
        .output()
        .with_context(|| format!("failed to execute python file: {}", file_name))?;

    if output.status.success() {
        let out = decode_str(&output.stdout)?;
        println!("----- result -----");
        println!("{}", out);
        println!("------ end -------");
    } else {
        let out = decode_str(&output.stderr)?;
        println!("----- error -----");
        println!("{}", out);
        println!("------ end -------");
    }
    Ok(())
}

fn decode_str(v: &[u8]) -> Result<String> {
    match String::from_utf8(v.to_vec()) {
        Ok(s) => Ok(s),
        Err(_) => {
            let (res, _, _) = encoding_rs::SHIFT_JIS.decode(v);
            Ok(res.into_owned())
        }
    }
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

    let exec_path = env::current_exe()?;
    let exec_dir = exec_path.parent().unwrap();

    let output = Command::new("python")
        .args(&[
            "-u",
            exec_dir.join("script/gen_type_stubs.py").to_str().unwrap(),
            package_name,
        ])
        .output()
        .with_context(|| format!("failed to execute python file: {}", package_name))?;

    if output.status.success() {
        println!("{}", str::from_utf8(&output.stdout).unwrap());
    } else {
        println!("{}", str::from_utf8(&output.stderr).unwrap());
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
