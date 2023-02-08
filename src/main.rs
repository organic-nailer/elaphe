use std::env;
use std::process::Command;
use std::str;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        println!("arg 1 is missing.");
        return;
    }
    let source = &args[1];
    let output = "main.pyc";
    elaphe::run(output, source);

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
}
