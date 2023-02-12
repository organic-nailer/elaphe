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
        elaphe::run(&output, "main() { print((1+2)*5); }").expect("execution failed.");
        exec_py_and_assert(&output, "15\n");

        elaphe::run(&output, "main() { print(5*(1-2)); }").expect("execution failed.");
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
        elaphe::run(&output, "main() { print(1 + 2.3); }").expect("execution failed.");
        exec_py_and_assert(&output, "3.3\n");
    
        elaphe::run(&output, "main() { print(.5 * 4e+2); }").expect("execution failed.");
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
        elaphe::run(&output, "main() { print(0x47 - 0X05); }").expect("execution failed.");
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
        elaphe::run(&output, "main() { print(true + false); }").expect("execution failed.");
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
        elaphe::run(&output, "main() { print('abc' + 'defg'); }").expect("execution failed.");
        exec_py_and_assert(&output, "abcdefg\n");
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
        elaphe::run(&output, "main() { print(1 == 2); }").expect("execution failed.");
        exec_py_and_assert(&output, "False\n");
        elaphe::run(&output, "main() { print(1 != 2); }").expect("execution failed.");
        exec_py_and_assert(&output, "True\n");
        elaphe::run(&output, "main() { print(1 >= 2); }").expect("execution failed.");
        exec_py_and_assert(&output, "False\n");
        elaphe::run(&output, "main() { print(1.3 < 2.1); }").expect("execution failed.");
        exec_py_and_assert(&output, "True\n");
        elaphe::run(&output, "main() { print(1 > 2); }").expect("execution failed.");
        exec_py_and_assert(&output, "False\n");
        elaphe::run(&output, "main() { print(1 <= 2); }").expect("execution failed.");
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
        elaphe::run(&output, "main() { print(1 << 2); }").expect("execution failed.");
        exec_py_and_assert(&output, "4\n");
        elaphe::run(&output, "main() { print(8 >> 2); }").expect("execution failed.");
        exec_py_and_assert(&output, "2\n");
        elaphe::run(&output, "main() { print(3 & 6); }").expect("execution failed.");
        exec_py_and_assert(&output, "2\n");
        elaphe::run(&output, "main() { print(3 | 6); }").expect("execution failed.");
        exec_py_and_assert(&output, "7\n");
        elaphe::run(&output, "main() { print(3 ^ 6); }").expect("execution failed.");
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
        elaphe::run(&output, "main() { print(1+-2); }").expect("execution failed.");
        exec_py_and_assert(&output, "-1\n");
        elaphe::run(&output, "main() { print(~2); }").expect("execution failed.");
        exec_py_and_assert(&output, "-3\n");
        elaphe::run(&output, "main() { print(!(1!=2)); }").expect("execution failed.");
        exec_py_and_assert(&output, "False\n");
    });
    clean(&output);
    if result.is_err() {
        panic!("{:?}", result);
    }
}

#[test]
fn statement_list() {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    let result = catch_unwind(|| {
        elaphe::run(&output, "main() { {print(1+2);print(3-4);} }").expect("execution failed.");
        exec_py_and_assert(&output, "3\n-1\n");
    });
    clean(&output);
    if result.is_err() {
        panic!("{:?}", result);
    }
}


#[test]
fn global_variable() {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    let result = catch_unwind(|| {
        elaphe::run(&output, "main() { {var x = 4;print(x*x);} }").expect("execution failed.");
        exec_py_and_assert(&output, "16\n");
    });
    clean(&output);
    if result.is_err() {
        panic!("{:?}", result);
    }
}

#[test]
fn if_statement() {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    let result = catch_unwind(|| {
        elaphe::run(&output, "
        main() {
            if (1 == 2) {
                print(1);
            }
            else if (2 == 3) {
                print(2);
            }
            else {
                print(3);
            }
        }
        ").expect("execution failed.");
        exec_py_and_assert(&output, "3\n");
    });
    clean(&output);
    if result.is_err() {
        panic!("{:?}", result);
    }
}

#[test]
fn for_statement() {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    let result = catch_unwind(|| {
        elaphe::run(&output, "
        main() {
            for (var i = 0; i < 5; i += 1) {
                print(i);
            }
        }
        ").expect("execution failed.");
        exec_py_and_assert(&output, "0\n1\n2\n3\n4\n");
    });
    clean(&output);
    if result.is_err() {
        panic!("{:?}", result);
    }
}

#[test]
fn while_statement() {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    let result = catch_unwind(|| {
        elaphe::run(&output, "
        main() {
            var i = -5;
            while(i < 0) {
                print(i);
                i += 1;
            }
        }
        ").expect("execution failed.");
        exec_py_and_assert(&output, "-5\n-4\n-3\n-2\n-1\n");
    });
    clean(&output);
    if result.is_err() {
        panic!("{:?}", result);
    }
}

#[test]
fn do_statement() {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    let result = catch_unwind(|| {
        elaphe::run(&output, "
        main() {
            var i = -5;
            do {
                print(i);
                i += 1;
            } while(i < 0);
        }
        ").expect("execution failed.");
        exec_py_and_assert(&output, "-5\n-4\n-3\n-2\n-1\n");
    });
    clean(&output);
    if result.is_err() {
        panic!("{:?}", result);
    }
}

#[test]
fn assignment_expressions() {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    let result = catch_unwind(|| {
        elaphe::run(&output, "main() { {var x = 4; x = 2; print(x);} }").expect("execution failed.");
        exec_py_and_assert(&output, "2\n");
        elaphe::run(&output, "main() { {var x = 4; x += 2; print(x);} }").expect("execution failed.");
        exec_py_and_assert(&output, "6\n");
        elaphe::run(&output, "main() { {var x = 4; x *= 2; print(x);} }").expect("execution failed.");
        exec_py_and_assert(&output, "8\n");
        elaphe::run(&output, "main() { {var x = 5; x /= 2; print(x);} }").expect("execution failed.");
        exec_py_and_assert(&output, "2.5\n");
        elaphe::run(&output, "main() { {var x = 5; x %= 2; print(x);} }").expect("execution failed.");
        exec_py_and_assert(&output, "1\n");
        elaphe::run(&output, "main() { {var x = 5; x ~/= 2; print(x);} }").expect("execution failed.");
        exec_py_and_assert(&output, "2\n");
        elaphe::run(&output, "main() { {var x = 4; x <<= 2; print(x);} }").expect("execution failed.");
        exec_py_and_assert(&output, "16\n");
        elaphe::run(&output, "main() { {var x = 4; x >>= 2; print(x);} }").expect("execution failed.");
        exec_py_and_assert(&output, "1\n");
        elaphe::run(&output, "main() { {var x = 3; x &= 6; print(x);} }").expect("execution failed.");
        exec_py_and_assert(&output, "2\n");
        elaphe::run(&output, "main() { {var x = 3; x ^= 6; print(x);} }").expect("execution failed.");
        exec_py_and_assert(&output, "5\n");
        elaphe::run(&output, "main() { {var x = 3; x |= 6; print(x);} }").expect("execution failed.");
        exec_py_and_assert(&output, "7\n");
        elaphe::run(&output, "main() { {var x = null; x ??= 2; print(x);} }").expect("execution failed.");
        exec_py_and_assert(&output, "2\n");
        elaphe::run(&output, "main() { {var x = 4; x ??= 2; print(x);} }").expect("execution failed.");
        exec_py_and_assert(&output, "4\n");
    });
    clean(&output);
    if result.is_err() {
        panic!("{:?}", result);
    }
}

#[test]
fn top_level_functions() {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    let result = catch_unwind(|| {
        elaphe::run(&output, "
        sub() => print(10);
        main() {
            print(1);
            sub();
            print(100);
        }
        ").expect("execution failed.");
        exec_py_and_assert(&output, "1\n10\n100\n");
    });
    clean(&output);
    if result.is_err() {
        panic!("{:?}", result);
    }
}

#[test]
fn top_level_variables() {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    let result = catch_unwind(|| {
        elaphe::run(&output, "
        var x = 1;

        sub() {
            x = 2;
        }
        
        main() {
            print(x);
            sub();
            print(x);
        }
        ").expect("execution failed.");
        exec_py_and_assert(&output, "1\n2\n");
    });
    clean(&output);
    if result.is_err() {
        panic!("{:?}", result);
    }
}
