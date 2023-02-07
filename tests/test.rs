use std::panic::catch_unwind;
use std::path::Path;
use std::process::Command;
use std::{fs, str};

use uuid::Uuid;

fn exec_py_and_assert(filename: &str, expect: &str) {
    let py_command = format!("python {}", filename);
    match Command::new("bash").args(&["-c", &py_command]).output() {
        Ok(r) => assert_eq!(expect, str::from_utf8(&r.stdout).unwrap()),
        Err(e) => panic!("Command Failed: {}", e.to_string()),
    }
}

fn clean(filename: &str) {
    let path = Path::new(filename);
    fs::remove_file(path).ok();
}

#[test]
fn calc_operations() {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    let result = catch_unwind(|| {
        elaphe::run(&output, "(1+2)*5");
        exec_py_and_assert(&output, "15\n");

        elaphe::run(&output, "5*(1-2)");
        exec_py_and_assert(&output, "-5\n");
    });
    clean(&output);
    if result.is_err() {
        panic!("{:?}", result);
    }
}

#[test]
fn calc_float() {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    let result = catch_unwind(|| {
        elaphe::run(&output, "1 + 2.3");
        exec_py_and_assert(&output, "3.3\n");
    
        elaphe::run(&output, ".5 * 4e+2");
        exec_py_and_assert(&output, "200.0\n");
    });
    clean(&output);
    if result.is_err() {
        panic!("{:?}", result);
    }
}

#[test]
fn calc_hex() {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    let result = catch_unwind(|| {
        elaphe::run(&output, "0x47 - 0X05");
        exec_py_and_assert(&output, "66\n");
    });
    clean(&output);
    if result.is_err() {
        panic!("{:?}", result);
    }
}

#[test]
fn calc_boolean() {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    let result = catch_unwind(|| {
        elaphe::run(&output, "true + false");
        exec_py_and_assert(&output, "1\n");    
    });
    clean(&output);
    if result.is_err() {
        panic!("{:?}", result);
    }
}

#[test]
fn concat_string() {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    let result = catch_unwind(|| {
        elaphe::run(&output, "'abc' + 'defg'");
        exec_py_and_assert(&output, "b'abcdefg'\n");
    });
    clean(&output);
    if result.is_err() {
        panic!("{:?}", result);
    }
}

#[test]
fn compare_op() {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    let result = catch_unwind(|| {
        elaphe::run(&output, "1 == 2");
        exec_py_and_assert(&output, "False\n");
        elaphe::run(&output, "1 != 2");
        exec_py_and_assert(&output, "True\n");
        elaphe::run(&output, "1 >= 2");
        exec_py_and_assert(&output, "False\n");
        elaphe::run(&output, "1.3 < 2.1");
        exec_py_and_assert(&output, "True\n");
        elaphe::run(&output, "1 > 2");
        exec_py_and_assert(&output, "False\n");
        elaphe::run(&output, "1 <= 2");
        exec_py_and_assert(&output, "True\n");
    });
    clean(&output);
    if result.is_err() {
        panic!("{:?}", result);
    }
}

#[test]
fn binary_op() {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    let result = catch_unwind(|| {
        elaphe::run(&output, "1 << 2");
        exec_py_and_assert(&output, "4\n");
        elaphe::run(&output, "8 >> 2");
        exec_py_and_assert(&output, "2\n");
        elaphe::run(&output, "3 & 6");
        exec_py_and_assert(&output, "2\n");
        elaphe::run(&output, "3 | 6");
        exec_py_and_assert(&output, "7\n");
        elaphe::run(&output, "3 ^ 6");
        exec_py_and_assert(&output, "5\n");
    });
    clean(&output);
    if result.is_err() {
        panic!("{:?}", result);
    }
}

#[test]
fn unary_op() {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    let result = catch_unwind(|| {
        elaphe::run(&output, "1+-2");
        exec_py_and_assert(&output, "-1\n");
        elaphe::run(&output, "~2");
        exec_py_and_assert(&output, "-3\n");
        elaphe::run(&output, "!(1!=2)");
        exec_py_and_assert(&output, "False\n");
    });
    clean(&output);
    if result.is_err() {
        panic!("{:?}", result);
    }
}