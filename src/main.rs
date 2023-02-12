use std::error::Error;
use std::{env, fs};
use std::process::Command;
use std::str;
use getopts::Options;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optopt("c", "", "eval string", "CODE");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => { panic!("{}", f.to_string()) }
    };

    // println!("free: {:?}", matches.free);
    // println!("option c: {:?}", matches.opt_str("c"));

    let output = "main.pyc";

    if !matches.free.is_empty() {
        // ファイル名で実行
        let file_name = matches.free[0].clone();
        let source = fs::read_to_string(file_name)?;
        // println!("source:\n{}", source);
        
        compile_and_run(output, &source)?;
        Ok(())
    }
    else {
        // 文字列を実行
        let source = matches.opt_str("c");
        match source {
            Some(source) => {
                compile_and_run(output, &source)?;
                Ok(())
            },
            None => {
                Err("invalid arguments".into())
            }
        }
    }
}

fn compile_and_run(output: &str, source: &str) -> Result<(), Box<dyn Error>> {
    match elaphe::run(output, &source) {
        Ok(_) => println!("{} is generated!", output),
        Err(_) => {
            return Err("".into());
        }
    }

    println!("run {}", output);
    match Command::new("bash")
        .args(&["-c", "python main.pyc"])
        .output()
    {
        Ok(e) => {
            println!("----- result -----");
            println!("{}", str::from_utf8(&e.stdout).unwrap());
            println!("------ end -------");
        },
        Err(e) => println!("Error: {}", e),
    }
    Ok(())
}
