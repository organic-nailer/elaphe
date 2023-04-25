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
        elaphe::build_from_code_single(&output, "main() { print((1+2)*5); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "15\n");

        elaphe::build_from_code_single(&output, "main() { print(5*(1-2)); }")
            .expect("execution failed.");
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
        elaphe::build_from_code_single(&output, "main() { print(1 + 2.3); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "3.3\n");

        elaphe::build_from_code_single(&output, "main() { print(.5 * 4e+2); }")
            .expect("execution failed.");
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
        elaphe::build_from_code_single(&output, "main() { print(0x47 - 0X05); }")
            .expect("execution failed.");
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
        elaphe::build_from_code_single(&output, "main() { print(true + false); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "1\n");
    });
    clean(&output);
    if result.is_err() {
        panic!("{:?}", result);
    }
}

#[test]
fn string_literal() {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    let result = catch_unwind(|| {
        elaphe::build_from_code_single(&output, "main() { print('abc' + 'defg'); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "abcdefg\n");
        elaphe::build_from_code_single(&output, "main() { print('abc' 'defg'); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "abcdefg\n");
        elaphe::build_from_code_single(&output, r#"main() { print('"world"'); }"#)
            .expect("execution failed.");
        exec_py_and_assert(&output, "\"world\"\n");
        elaphe::build_from_code_single(&output, r#"main() { print("'world'"); }"#)
            .expect("execution failed.");
        exec_py_and_assert(&output, "'world'\n");
        elaphe::build_from_code_single(
            &output,
            r#"
        main() {
            print('''
Hello,
World!
''');
        }
        "#,
        )
        .expect("execution failed.");
        exec_py_and_assert(&output, "Hello,\nWorld!\n\n");
        elaphe::build_from_code_single(
            &output,
            r#"
        main() {
            print("""     
Hello,
World!
""");
        }
        "#,
        )
        .expect("execution failed.");
        exec_py_and_assert(&output, "Hello,\nWorld!\n\n");
    });
    clean(&output);
    if result.is_err() {
        panic!("{:?}", result);
    }
}

#[test]
fn string_interpolation() {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    let result = catch_unwind(|| {
        elaphe::build_from_code_single(
            &output,
            "main() { var x = 'world!'; print('Hello, ${x}'); }",
        )
        .expect("execution failed.");
        exec_py_and_assert(&output, "Hello, world!\n");
        elaphe::build_from_code_single(&output, "main() { print('1+1=${1+1}'); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "1+1=2\n");
        elaphe::build_from_code_single(
            &output,
            r#"main() { var x = "elaphe"; print("Hello, ${x} and ${"dart"}!"); }"#,
        )
        .expect("execution failed.");
        exec_py_and_assert(&output, "Hello, elaphe and dart!\n");
        elaphe::build_from_code_single(
            &output,
            r#"main() { var x = "recursive"; print("This is ${'${x} interpolation'}."); }"#,
        )
        .expect("execution failed.");
        exec_py_and_assert(&output, "This is recursive interpolation.\n");
        elaphe::build_from_code_single(
            &output,
            "main() { var a = 'Hello'; var b = 'world'; print('$a, $b!'); }",
        )
        .expect("execution failed.");
        exec_py_and_assert(&output, "Hello, world!\n");
    });
    clean(&output);
    if result.is_err() {
        panic!("{:?}", result);
    }
}

#[test]
fn string_escape() {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    let result = catch_unwind(|| {
        elaphe::build_from_code_single(&output, r#"main() { print('\n\r\f\b\t\v'); }"#)
            .expect("execution failed.");
        exec_py_and_assert(&output, "\n\r\x0C\x08\t\x0B\n");
        elaphe::build_from_code_single(
            &output,
            r#"main() { print('\x4B\u{4f}\u6176\u{0061C9}'); }"#,
        )
        .expect("execution failed.");
        exec_py_and_assert(&output, "KO慶應\n");
        elaphe::build_from_code_single(&output, r#"main() { print('\a\c\'\\\$'); }"#)
            .expect("execution failed.");
        exec_py_and_assert(&output, "ac'\\$\n");
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
        elaphe::build_from_code_single(&output, "main() { print(1 == 2); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "False\n");
        elaphe::build_from_code_single(&output, "main() { print(1 != 2); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "True\n");
        elaphe::build_from_code_single(&output, "main() { print(1 >= 2); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "False\n");
        elaphe::build_from_code_single(&output, "main() { print(1.3 < 2.1); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "True\n");
        elaphe::build_from_code_single(&output, "main() { print(1 > 2); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "False\n");
        elaphe::build_from_code_single(&output, "main() { print(1 <= 2); }")
            .expect("execution failed.");
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
        elaphe::build_from_code_single(&output, "main() { print(1 << 2); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "4\n");
        elaphe::build_from_code_single(&output, "main() { print(8 >> 2); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "2\n");
        elaphe::build_from_code_single(&output, "main() { print(3 & 6); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "2\n");
        elaphe::build_from_code_single(&output, "main() { print(3 | 6); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "7\n");
        elaphe::build_from_code_single(&output, "main() { print(3 ^ 6); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "5\n");
        elaphe::build_from_code_single(&output, "main() { print(5 + 2); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "7\n");
        elaphe::build_from_code_single(&output, "main() { print(5 - 2); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "3\n");
        elaphe::build_from_code_single(&output, "main() { print(5 * 2); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "10\n");
        elaphe::build_from_code_single(&output, "main() { print(5 / 2); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "2.5\n");
        elaphe::build_from_code_single(&output, "main() { print(5 ~/ 2); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "2\n");
        elaphe::build_from_code_single(&output, "main() { print(5 % 2); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "1\n");
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
        elaphe::build_from_code_single(&output, "main() { print(1+-2); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "-1\n");
        elaphe::build_from_code_single(&output, "main() { print(~2); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "-3\n");
        elaphe::build_from_code_single(&output, "main() { print(!(1!=2)); }")
            .expect("execution failed.");
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
        elaphe::build_from_code_single(&output, "main() { {print(1+2);print(3-4);} }")
            .expect("execution failed.");
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
        elaphe::build_from_code_single(&output, "var x = 4; main() { {print(x*x);} }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "16\n");
    });
    clean(&output);
    if result.is_err() {
        panic!("{:?}", result);
    }
}

#[test]
fn local_variable() {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    let result = catch_unwind(|| {
        elaphe::build_from_code_single(&output, "main() { var x = 4; {print(x*x);} }")
            .expect("execution failed.");
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
        elaphe::build_from_code_single(
            &output,
            "
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
        ",
        )
        .expect("execution failed.");
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
        elaphe::build_from_code_single(
            &output,
            "
        main() {
            for (var i = 0; i < 5; i += 1) {
                print(i);
            }
        }
        ",
        )
        .expect("execution failed.");
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
        elaphe::build_from_code_single(
            &output,
            "
        main() {
            var i = -5;
            while(i < 0) {
                print(i);
                i += 1;
            }
        }
        ",
        )
        .expect("execution failed.");
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
        elaphe::build_from_code_single(
            &output,
            "
        main() {
            var i = -5;
            do {
                print(i);
                i += 1;
            } while(i < 0);
        }
        ",
        )
        .expect("execution failed.");
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
        elaphe::build_from_code_single(&output, "main() { var x = 4; x = 10; print(x); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "10\n");
        elaphe::build_from_code_single(&output, "main() { var x = 4; print(x = 2); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "2\n");
        elaphe::build_from_code_single(&output, "main() { var x = 4; print(x += 2); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "6\n");
        elaphe::build_from_code_single(&output, "main() { var x = 4; x *= 2; print(x); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "8\n");
        elaphe::build_from_code_single(&output, "main() { var x = 5; x /= 2; print(x); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "2.5\n");
        elaphe::build_from_code_single(&output, "main() { var x = 5; x %= 2; print(x); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "1\n");
        elaphe::build_from_code_single(&output, "main() { var x = 5; x ~/= 2; print(x); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "2\n");
        elaphe::build_from_code_single(&output, "main() { var x = 4; x <<= 2; print(x); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "16\n");
        elaphe::build_from_code_single(&output, "main() { var x = 4; x >>= 2; print(x); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "1\n");
        elaphe::build_from_code_single(&output, "main() { var x = 3; x &= 6; print(x); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "2\n");
        elaphe::build_from_code_single(&output, "main() { var x = 3; x ^= 6; print(x); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "5\n");
        elaphe::build_from_code_single(&output, "main() { var x = 3; x |= 6; print(x); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "7\n");
        elaphe::build_from_code_single(&output, "main() { var x = null; x ??= 2; print(x); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "2\n");
        elaphe::build_from_code_single(&output, "main() { var x = 4; x ??= 2; print(x); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "4\n");
        elaphe::build_from_code_single(&output, "main() { var x = null; print(x ??= 2); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "2\n");
        elaphe::build_from_code_single(&output, "main() { var x = 4; print(x ??= 2); }")
            .expect("execution failed.");
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
        elaphe::build_from_code_single(
            &output,
            "
        sub() => print(10);
        main() {
            print(1);
            sub();
            print(100);
        }
        ",
        )
        .expect("execution failed.");
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
        elaphe::build_from_code_single(
            &output,
            "
        var x = 1;

        sub() {
            x = 2;
        }
        
        main() {
            print(x);
            sub();
            print(x);
        }
        ",
        )
        .expect("execution failed.");
        exec_py_and_assert(&output, "1\n2\n");
    });
    clean(&output);
    if result.is_err() {
        panic!("{:?}", result);
    }
}

#[test]
fn function_with_arguments() {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    let result = catch_unwind(|| {
        elaphe::build_from_code_single(
            &output,
            "
        add(a,b) {
            print(a+b);
        }
        
        main() {
            add(10,100);
            add(200,-200);
        }
        ",
        )
        .expect("execution failed.");
        exec_py_and_assert(&output, "110\n0\n");
    });
    clean(&output);
    if result.is_err() {
        panic!("{:?}", result);
    }
}

#[test]
fn import_libraries() {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    let result = catch_unwind(|| {
        elaphe::build_from_code_single(
            &output,
            "
        import 'elaphe/math.d.dart';

        main() {
            var x = math.sqrt(4);
            print(x);
            var y = math.floor(math.pi);
            print(y);
        }
        ",
        )
        .expect("execution failed.");
        exec_py_and_assert(&output, "2.0\n3\n");
    });
    clean(&output);
    if result.is_err() {
        panic!("{:?}", result);
    }
}

#[test]
fn conditional_expression() {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    let result = catch_unwind(|| {
        elaphe::build_from_code_single(&output, "main() { var x = 1; print(x == 2 ? 10 : 20); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "20\n");
        elaphe::build_from_code_single(&output, "main() { var x = 2; print(x == 2 ? 10 : 20); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "10\n");
    });
    clean(&output);
    if result.is_err() {
        panic!("{:?}", result);
    }
}

#[test]
fn logical_expression() {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    let result = catch_unwind(|| {
        elaphe::build_from_code_single(&output, "main() { var x = 1; print(x == 1 && x == 2); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "False\n");
        elaphe::build_from_code_single(&output, "main() { var x = 2; print(x == 1 || x == 2); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "True\n");
        elaphe::build_from_code_single(&output, "main() { var x = null; print(x ?? 10); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "10\n");
        elaphe::build_from_code_single(&output, "main() { var x = 1; print(x ?? 10); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "1\n");
    });
    clean(&output);
    if result.is_err() {
        panic!("{:?}", result);
    }
}

#[test]
fn loop_label() {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    let result = catch_unwind(|| {
        elaphe::build_from_code_single(
            &output,
            "
        main() { 
            outerloop:
               
            for (var i = 0; i < 5; i += 1) { 
                print(i * 10); 
                innerloop: 
                for (var j = 0; j < 5; j += 1) { 
                    if (j > 3 ) break ; 
                     
                    if (i == 2) break innerloop; 
                    
                    if (i == 4) break outerloop; 
                     
                    print(j); 
                } 
            } 
        }
        ",
        )
        .expect("execution failed.");
        exec_py_and_assert(
            &output,
            "0\n0\n1\n2\n3\n10\n0\n1\n2\n3\n20\n30\n0\n1\n2\n3\n40\n",
        );
    });
    clean(&output);
    if result.is_err() {
        panic!("{:?}", result);
    }
}

#[test]
fn comment() {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    let result = catch_unwind(|| {
        elaphe::build_from_code_single(
            &output,
            "
        main() { 
            var x = 1;// single comment
            var y /* inner comment */ = 1;
            /*
            multi
            line
            comment
            print(x + y);
            */ print(x * y); /* comment */
        }
        ",
        )
        .expect("execution failed.");
        exec_py_and_assert(&output, "1\n");
    });
    clean(&output);
    if result.is_err() {
        panic!("{:?}", result);
    }
}

#[test]
fn return_value() {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    let result = catch_unwind(|| {
        elaphe::build_from_code_single(
            &output,
            "
        add(x,y) {
            return x+y;
        }
        
        main() { 
            print(add(10,20));
        }
        ",
        )
        .expect("execution failed.");
        exec_py_and_assert(&output, "30\n");
    });
    clean(&output);
    if result.is_err() {
        panic!("{:?}", result);
    }
}

#[test]
fn switch_statement() {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    let result = catch_unwind(|| {
        elaphe::build_from_code_single(
            &output,
            r#"
        main() { 
            var x = 1;
            switch (x) {
                case 0:
                  print("zero");
                  break;
                case 1:
                case 2:
                  print("one or two");
                  break;
                default:
                  print("more");
                  break;
            }
        }
        "#,
        )
        .expect("execution failed.");
        exec_py_and_assert(&output, "one or two\n");
    });
    clean(&output);
    if result.is_err() {
        panic!("{:?}", result);
    }
}

#[test]
fn update_expression() {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    let result = catch_unwind(|| {
        elaphe::build_from_code_single(&output, "main() { var x = 1; print(x++); print(x); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "1\n2\n");
        elaphe::build_from_code_single(&output, "main() { var x = 1; print(x--); print(x); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "1\n0\n");
        elaphe::build_from_code_single(&output, "main() { var x = 1; print(++x); print(x); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "2\n2\n");
        elaphe::build_from_code_single(&output, "main() { var x = 1; print(--x); print(x); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "0\n0\n");
    });
    clean(&output);
    if result.is_err() {
        panic!("{:?}", result);
    }
}

#[test]
fn try_statement() {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    let result = catch_unwind(|| {
        elaphe::build_from_code_single(
            &output,
            r#"
        main() {
            try {
              print("try");
              throw IOError;
            }
            on IOError catch(e,t) {
              print("IOError");
            }
            catch(e) {
              print("Unknown");
            }
            finally {
              print("finally");
            }
        }
        "#,
        )
        .expect("execution failed.");
        exec_py_and_assert(&output, "try\nIOError\nfinally\n");
        elaphe::build_from_code_single(
            &output,
            r#"
        main() {
            try {
              print("try");
              throw KeyboardInterrupt;
            }
            on IOError {
              print("IOError");
            }
            catch(e) {
              print("Unknown");
            }
            finally {
              print("finally");
            }
        }
        "#,
        )
        .expect("execution failed.");
        exec_py_and_assert(&output, "try\nUnknown\nfinally\n");
    });
    clean(&output);
    if result.is_err() {
        panic!("{:?}", result);
    }
}

#[test]
fn collection_literal() {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    let result = catch_unwind(|| {
        elaphe::build_from_code_single(&output, "main() { print([1,2,3]); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "[1, 2, 3]\n");
        elaphe::build_from_code_single(&output, "main() { print({1,2,3}); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "{1, 2, 3}\n");
        elaphe::build_from_code_single(&output, "main() { print({'a':1, 'b':2}); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "{'a': 1, 'b': 2}\n");
    });
    clean(&output);
    if result.is_err() {
        panic!("{:?}", result);
    }
}

#[test]
fn subscr() {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    let result = catch_unwind(|| {
        elaphe::build_from_code_single(&output, "main() { var x = [0,1,2]; print(x[2]); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "2\n");
        elaphe::build_from_code_single(
            &output,
            "main() { var x = [0,1,2]; print(x[2] = 10); print(x); }",
        )
        .expect("execution failed.");
        exec_py_and_assert(&output, "10\n[0, 1, 10]\n");
        elaphe::build_from_code_single(
            &output,
            "main() { var x = [0,1,2]; print(x[2] += 10); print(x); }",
        )
        .expect("execution failed.");
        exec_py_and_assert(&output, "12\n[0, 1, 12]\n");
        elaphe::build_from_code_single(
            &output,
            "main() { var x = [0,1,null]; print(x[2] ??= 10); print(x); }",
        )
        .expect("execution failed.");
        exec_py_and_assert(&output, "10\n[0, 1, 10]\n");
        elaphe::build_from_code_single(
            &output,
            "main() { var x = [0,1,2]; print(x[2] ??= 10); print(x); }",
        )
        .expect("execution failed.");
        exec_py_and_assert(&output, "2\n[0, 1, 2]\n");
    });
    clean(&output);
    if result.is_err() {
        panic!("{:?}", result);
    }
}

#[test]
fn type_annotation() {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    let result = catch_unwind(|| {
        elaphe::build_from_code_single(&output, "main() { int x = 1; print(x); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "1\n");
        elaphe::build_from_code_single(&output, "main() { final int x = 1; print(x); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "1\n");
        elaphe::build_from_code_single(&output, "main() { final x = 1; print(x); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "1\n");
        elaphe::build_from_code_single(&output, "main() { const x = 1; print(x); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "1\n");
        elaphe::build_from_code_single(&output, "main() { const int x = 1; print(x); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "1\n");
        // elaphe::build_from_code_single(&output, "main() { np.int32 x = 1; print(x); }").expect("execution failed.");
        exec_py_and_assert(&output, "1\n");
        elaphe::build_from_code_single(&output, "main() { late final int x = 1; print(x); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "1\n");
    });
    clean(&output);
    if result.is_err() {
        panic!("{:?}", result);
    }
}

#[test]
fn variable_declaration() {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    let result = catch_unwind(|| {
        elaphe::build_from_code_single(&output, "main() { int x = 1, y = 2; print(x + y); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "3\n");
        elaphe::build_from_code_single(&output, "int x = 1, y = 2; main() { print(x + y); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "3\n");
    });
    clean(&output);
    if result.is_err() {
        panic!("{:?}", result);
    }
}

#[test]
fn function_parameters() {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    let result = catch_unwind(|| {
        elaphe::build_from_code_single(
            &output,
            r#"
        hoge1(int x, int y) {
            print(x + y);
        }
          
        hoge2(int x, [int y = 10]) {
            print(x + y);
        }
          
        hoge3(int x, {int y = 10}) {
            print(x + y);
        }
          
        main() {
            hoge1(1,2);
            hoge2(1);
            hoge2(1,2);
            hoge3(1);
            hoge3(1,y:100);
        }
        "#,
        )
        .expect("execution failed.");
        exec_py_and_assert(&output, "3\n11\n3\n11\n101\n");
    });
    clean(&output);
    if result.is_err() {
        panic!("{:?}", result);
    }
}

#[test]
fn class_method() {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    let result = catch_unwind(|| {
        elaphe::build_from_code_single(
            &output,
            r#"
        class Hoge {
            void greeting() {
              print("Hello!");
            }
        }
          
        void main() {
            Hoge hoge = Hoge();
            hoge.greeting();
        }
        "#,
        )
        .expect("execution failed.");
        exec_py_and_assert(&output, "Hello!\n");
    });
    clean(&output);
    if result.is_err() {
        panic!("{:?}", result);
    }
}

#[test]
fn class_field() {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    let result = catch_unwind(|| {
        elaphe::build_from_code_single(
            &output,
            r#"
        class Hoge {
            int x = 1;
            void greeting() {
              print("Hello!");
              print(x); // 1
              x = 2;
              print(x); // 2
              this.x = 3;
              print(x); // 3
            }
          
            void greeting2(int x) {
              print(x); // 10
              print(this.x); // 3
              x = 4;
              print(x); // 4
              print(this.x); // 3
              this.x = 5;
              print(x); // 4
              print(this.x); // 5
            }
        }
          
        void main() {
            Hoge hoge = Hoge();
            hoge.greeting();
            hoge.greeting2(10);
        }          
        "#,
        )
        .expect("execution failed.");
        exec_py_and_assert(&output, "Hello!\n1\n2\n3\n10\n3\n4\n3\n4\n5\n");
    });
    clean(&output);
    if result.is_err() {
        panic!("{:?}", result);
    }
}

#[test]
fn class_constructor() {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    let result = catch_unwind(|| {
        elaphe::build_from_code_single(
            &output,
            r#"
        class Hoge {
            int y = 0;
            Hoge(int x) {
              print(x);
              print(y);
              y = x;
              print(x);
              print(y);
            }
          }
          
          void main() {
            var h = Hoge(10);
          }
        "#,
        )
        .expect("execution failed.");
        exec_py_and_assert(&output, "10\n0\n10\n10\n");
    });
    clean(&output);
    if result.is_err() {
        panic!("{:?}", result);
    }
}

#[test]
fn slice() {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    let result = catch_unwind(|| {
        elaphe::build_from_code_single(
            &output,
            r#"
        void main() {
            var list = [0,1,2,3,4,5,6,7,8,9];
          
            print(list[sl()]);
            print(list[sl(7)]);
            print(list[sl(3,6)]);
            print(list[sl(null,3)]);
            print(list[sl(null,null,2)]);
            print(list[sl(3,null,3)]);
            print(list[sl(null,6,3)]);
            print(list[sl(3,6,2)]);
        }
        "#,
        )
        .expect("execution failed.");
        exec_py_and_assert(&output, "[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]\n[7, 8, 9]\n[3, 4, 5]\n[0, 1, 2]\n[0, 2, 4, 6, 8]\n[3, 6, 9]\n[0, 3]\n[3, 5]\n");
    });
    clean(&output);
    if result.is_err() {
        panic!("{:?}", result);
    }
}

#[test]
fn type_as_is() {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    let result = catch_unwind(|| {
        elaphe::build_from_code_single(&output, "main() { print(3 is int); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "True\n");
        elaphe::build_from_code_single(&output, "main() { print(2 is! int); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "False\n");
        elaphe::build_from_code_single(&output, "main() { var x = 1; print(x as int); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "1\n");
        elaphe::build_from_code_single(&output, "main() { var as = 1; print(as); }")
            .expect("execution failed.");
        exec_py_and_assert(&output, "1\n");
    });
    clean(&output);
    if result.is_err() {
        panic!("{:?}", result);
    }
}
