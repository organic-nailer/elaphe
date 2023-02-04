use std::process::Command;
use std::str;

fn exec_py_and_assert(filename: &str, expect: &str) {
    let py_command = format!("python {}", filename);
    match Command::new("bash")
        .args(&["-c", &py_command])
        .output()
    {
        Ok(r) => assert_eq!(expect,str::from_utf8(&r.stdout).unwrap()),
        Err(e) => panic!("Command Failed: {}", e.to_string()),
    }
}

#[test]
fn calc_operations() {
    elaphe::run("test.pyc", "(1+2)*5");
    exec_py_and_assert("test.pyc", "15\n");

    elaphe::run("test.pyc", "5*(1-2)");
    exec_py_and_assert("test.pyc", "-5\n");
}
