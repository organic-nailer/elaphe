use std::path::Path;
use std::process::Command;
use std::{fs, str};

use anyhow::{bail, Context, Result};
use uuid::Uuid;

fn exec_py_and_assert(filename: &str, expect: &str) -> Result<()> {
    let py_command = format!("python {}", filename);
    let output = Command::new("bash")
        .args(&["-c", &py_command])
        .output()
        .with_context(|| format!("failed to execute {}", py_command))?;

    match output.status.code() {
        Some(code) => {
            if code != 0 {
                let stderr = str::from_utf8(&output.stderr)
                    .with_context(|| format!("failed to parse stderr {}", py_command))?;
                bail!("failed to execute. code: {}, stderr:\n {}", code, stderr);
            }
        }
        None => bail!("failed to execute {}", py_command),
    }

    let stdout = str::from_utf8(&output.stdout)
        .with_context(|| format!("failed to parse stdout {}", py_command))?;
    assert_eq!(expect, stdout);
    Ok(())
}

fn clean(filename: &str) {
    let path = Path::new(filename);
    fs::remove_file(path).ok();
}

#[test]
fn calc_operations() -> Result<()> {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    elaphe::build_from_code_single(&output, "main() { print((1+2)*5); }")?;
    exec_py_and_assert(&output, "15\n")?;
    elaphe::build_from_code_single(&output, "main() { print(5*(1-2)); }")?;
    exec_py_and_assert(&output, "-5\n")?;

    clean(&output);
    Ok(())
}

#[test]
fn calc_float() -> Result<()> {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    elaphe::build_from_code_single(&output, "main() { print(1 + 2.3); }")?;
    exec_py_and_assert(&output, "3.3\n")?;

    elaphe::build_from_code_single(&output, "main() { print(.5 * 4e+2); }")?;
    exec_py_and_assert(&output, "200.0\n")?;
    clean(&output);
    Ok(())
}

#[test]
fn calc_hex() -> Result<()> {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    elaphe::build_from_code_single(&output, "main() { print(0x47 - 0X05); }")?;
    exec_py_and_assert(&output, "66\n")?;
    clean(&output);
    Ok(())
}

#[test]
fn calc_boolean() -> Result<()> {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    elaphe::build_from_code_single(&output, "main() { print(true + false); }")?;
    exec_py_and_assert(&output, "1\n")?;
    clean(&output);
    Ok(())
}

#[test]
fn string_literal() -> Result<()> {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    elaphe::build_from_code_single(&output, "main() { print('abc' + 'defg'); }")?;
    exec_py_and_assert(&output, "abcdefg\n")?;
    elaphe::build_from_code_single(&output, "main() { print('abc' 'defg'); }")?;
    exec_py_and_assert(&output, "abcdefg\n")?;
    elaphe::build_from_code_single(&output, r#"main() { print('"world"'); }"#)?;
    exec_py_and_assert(&output, "\"world\"\n")?;
    elaphe::build_from_code_single(&output, r#"main() { print("'world'"); }"#)?;
    exec_py_and_assert(&output, "'world'\n")?;
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
    )?;
    exec_py_and_assert(&output, "Hello,\nWorld!\n\n")?;
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
    )?;
    exec_py_and_assert(&output, "Hello,\nWorld!\n\n")?;
    clean(&output);
    Ok(())
}

#[test]
fn string_interpolation() -> Result<()> {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    elaphe::build_from_code_single(
        &output,
        "main() { var x = 'world!'; print('Hello, ${x}'); }",
    )?;
    exec_py_and_assert(&output, "Hello, world!\n")?;
    elaphe::build_from_code_single(&output, "main() { print('1+1=${1+1}'); }")?;
    exec_py_and_assert(&output, "1+1=2\n")?;
    elaphe::build_from_code_single(
        &output,
        r#"main() { var x = "elaphe"; print("Hello, ${x} and ${"dart"}!"); }"#,
    )?;
    exec_py_and_assert(&output, "Hello, elaphe and dart!\n")?;
    elaphe::build_from_code_single(
        &output,
        r#"main() { var x = "recursive"; print("This is ${'${x} interpolation'}."); }"#,
    )?;
    exec_py_and_assert(&output, "This is recursive interpolation.\n")?;
    elaphe::build_from_code_single(
        &output,
        "main() { var a = 'Hello'; var b = 'world'; print('$a, $b!'); }",
    )?;
    exec_py_and_assert(&output, "Hello, world!\n")?;
    clean(&output);
    Ok(())
}

#[test]
fn string_escape() -> Result<()> {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    elaphe::build_from_code_single(&output, r#"main() { print('\n\r\f\b\t\v'); }"#)?;
    exec_py_and_assert(&output, "\n\r\x0C\x08\t\x0B\n")?;
    elaphe::build_from_code_single(
        &output,
        r#"main() { print('\x4B\u{4f}\u6176\u{0061C9}'); }"#,
    )?;
    exec_py_and_assert(&output, "KO慶應\n")?;
    elaphe::build_from_code_single(&output, r#"main() { print('\a\c\'\\\$'); }"#)?;
    exec_py_and_assert(&output, "ac'\\$\n")?;
    clean(&output);
    Ok(())
}

#[test]
fn compare_op() -> Result<()> {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    elaphe::build_from_code_single(&output, "main() { print(1 == 2); }")?;
    exec_py_and_assert(&output, "False\n")?;
    elaphe::build_from_code_single(&output, "main() { print(1 != 2); }")?;
    exec_py_and_assert(&output, "True\n")?;
    elaphe::build_from_code_single(&output, "main() { print(1 >= 2); }")?;
    exec_py_and_assert(&output, "False\n")?;
    elaphe::build_from_code_single(&output, "main() { print(1.3 < 2.1); }")?;
    exec_py_and_assert(&output, "True\n")?;
    elaphe::build_from_code_single(&output, "main() { print(1 > 2); }")?;
    exec_py_and_assert(&output, "False\n")?;
    elaphe::build_from_code_single(&output, "main() { print(1 <= 2); }")?;
    exec_py_and_assert(&output, "True\n")?;
    clean(&output);
    Ok(())
}

#[test]
fn binary_op() -> Result<()> {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    elaphe::build_from_code_single(&output, "main() { print(1 << 2); }")?;
    exec_py_and_assert(&output, "4\n")?;
    elaphe::build_from_code_single(&output, "main() { print(8 >> 2); }")?;
    exec_py_and_assert(&output, "2\n")?;
    elaphe::build_from_code_single(&output, "main() { print(3 & 6); }")?;
    exec_py_and_assert(&output, "2\n")?;
    elaphe::build_from_code_single(&output, "main() { print(3 | 6); }")?;
    exec_py_and_assert(&output, "7\n")?;
    elaphe::build_from_code_single(&output, "main() { print(3 ^ 6); }")?;
    exec_py_and_assert(&output, "5\n")?;
    elaphe::build_from_code_single(&output, "main() { print(5 + 2); }")?;
    exec_py_and_assert(&output, "7\n")?;
    elaphe::build_from_code_single(&output, "main() { print(5 - 2); }")?;
    exec_py_and_assert(&output, "3\n")?;
    elaphe::build_from_code_single(&output, "main() { print(5 * 2); }")?;
    exec_py_and_assert(&output, "10\n")?;
    elaphe::build_from_code_single(&output, "main() { print(5 / 2); }")?;
    exec_py_and_assert(&output, "2.5\n")?;
    elaphe::build_from_code_single(&output, "main() { print(5 ~/ 2); }")?;
    exec_py_and_assert(&output, "2\n")?;
    elaphe::build_from_code_single(&output, "main() { print(5 % 2); }")?;
    exec_py_and_assert(&output, "1\n")?;
    clean(&output);
    Ok(())
}

#[test]
fn unary_op() -> Result<()> {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    elaphe::build_from_code_single(&output, "main() { print(1+-2); }")?;
    exec_py_and_assert(&output, "-1\n")?;
    elaphe::build_from_code_single(&output, "main() { print(~2); }")?;
    exec_py_and_assert(&output, "-3\n")?;
    elaphe::build_from_code_single(&output, "main() { print(!(1!=2)); }")?;
    exec_py_and_assert(&output, "False\n")?;
    clean(&output);
    Ok(())
}

#[test]
fn statement_list() -> Result<()> {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    elaphe::build_from_code_single(&output, "main() { {print(1+2);print(3-4);} }")?;
    exec_py_and_assert(&output, "3\n-1\n")?;
    clean(&output);
    Ok(())
}

#[test]
fn global_variable() -> Result<()> {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    elaphe::build_from_code_single(&output, "var x = 4; main() { {print(x*x);} }")?;
    exec_py_and_assert(&output, "16\n")?;
    clean(&output);
    Ok(())
}

#[test]
fn local_variable() -> Result<()> {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    elaphe::build_from_code_single(&output, "main() { var x = 4; {print(x*x);} }")?;
    exec_py_and_assert(&output, "16\n")?;
    clean(&output);
    Ok(())
}

#[test]
fn if_statement() -> Result<()> {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
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
    )?;
    exec_py_and_assert(&output, "3\n")?;
    clean(&output);
    Ok(())
}

#[test]
fn for_statement() -> Result<()> {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    elaphe::build_from_code_single(
        &output,
        "
        main() {
            for (var i = 0; i < 5; i += 1) {
                print(i);
            }
        }
        ",
    )?;
    exec_py_and_assert(&output, "0\n1\n2\n3\n4\n")?;
    clean(&output);
    Ok(())
}

#[test]
fn for_in_statement() -> Result<()> {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    elaphe::build_from_code_single(
        &output,
        "
        main() {
            var i;
            for (i in [0, 1, 2, 3, 4]) {
                print(i);
            }
        }
        ",
    )?;
    exec_py_and_assert(&output, "0\n1\n2\n3\n4\n")?;
    elaphe::build_from_code_single(
        &output,
        "
        main() {
            for (var i in [0, 1]) {
                print(i);
            }
        }
        ",
    )?;
    exec_py_and_assert(&output, "0\n1\n")?;
    clean(&output);
    Ok(())
}

#[test]
fn while_statement() -> Result<()> {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
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
    )?;
    exec_py_and_assert(&output, "-5\n-4\n-3\n-2\n-1\n")?;
    clean(&output);
    Ok(())
}

#[test]
fn do_statement() -> Result<()> {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
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
    )?;
    exec_py_and_assert(&output, "-5\n-4\n-3\n-2\n-1\n")?;
    clean(&output);
    Ok(())
}

#[test]
fn assignment_expressions() -> Result<()> {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    elaphe::build_from_code_single(&output, "main() { var x = 4; x = 10; print(x); }")?;
    exec_py_and_assert(&output, "10\n")?;
    elaphe::build_from_code_single(&output, "main() { var x = 4; print(x = 2); }")?;
    exec_py_and_assert(&output, "2\n")?;
    elaphe::build_from_code_single(&output, "main() { var x = 4; print(x += 2); }")?;
    exec_py_and_assert(&output, "6\n")?;
    elaphe::build_from_code_single(&output, "main() { var x = 4; x *= 2; print(x); }")?;
    exec_py_and_assert(&output, "8\n")?;
    elaphe::build_from_code_single(&output, "main() { var x = 5; x /= 2; print(x); }")?;
    exec_py_and_assert(&output, "2.5\n")?;
    elaphe::build_from_code_single(&output, "main() { var x = 5; x %= 2; print(x); }")?;
    exec_py_and_assert(&output, "1\n")?;
    elaphe::build_from_code_single(&output, "main() { var x = 5; x ~/= 2; print(x); }")?;
    exec_py_and_assert(&output, "2\n")?;
    elaphe::build_from_code_single(&output, "main() { var x = 4; x <<= 2; print(x); }")?;
    exec_py_and_assert(&output, "16\n")?;
    elaphe::build_from_code_single(&output, "main() { var x = 4; x >>= 2; print(x); }")?;
    exec_py_and_assert(&output, "1\n")?;
    elaphe::build_from_code_single(&output, "main() { var x = 3; x &= 6; print(x); }")?;
    exec_py_and_assert(&output, "2\n")?;
    elaphe::build_from_code_single(&output, "main() { var x = 3; x ^= 6; print(x); }")?;
    exec_py_and_assert(&output, "5\n")?;
    elaphe::build_from_code_single(&output, "main() { var x = 3; x |= 6; print(x); }")?;
    exec_py_and_assert(&output, "7\n")?;
    elaphe::build_from_code_single(&output, "main() { var x = null; x ??= 2; print(x); }")?;
    exec_py_and_assert(&output, "2\n")?;
    elaphe::build_from_code_single(&output, "main() { var x = 4; x ??= 2; print(x); }")?;
    exec_py_and_assert(&output, "4\n")?;
    elaphe::build_from_code_single(&output, "main() { var x = null; print(x ??= 2); }")?;
    exec_py_and_assert(&output, "2\n")?;
    elaphe::build_from_code_single(&output, "main() { var x = 4; print(x ??= 2); }")?;
    exec_py_and_assert(&output, "4\n")?;
    clean(&output);
    Ok(())
}

#[test]
fn top_level_functions() -> Result<()> {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
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
    )?;
    exec_py_and_assert(&output, "1\n10\n100\n")?;
    clean(&output);
    Ok(())
}

#[test]
fn top_level_variables() -> Result<()> {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
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
    )?;
    exec_py_and_assert(&output, "1\n2\n")?;
    clean(&output);
    Ok(())
}

#[test]
fn function_with_arguments() -> Result<()> {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
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
    )?;
    exec_py_and_assert(&output, "110\n0\n")?;
    clean(&output);
    Ok(())
}

#[test]
fn import_libraries() -> Result<()> {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    elaphe::build_from_code_single(
        &output,
        "
        import 'elaphe/math.d.dart';

        main() {
            var x = sqrt(4);
            print(x);
            var y = floor(pi);
            print(y);
        }
        ",
    )?;
    exec_py_and_assert(&output, "2.0\n3\n")?;
    clean(&output);
    Ok(())
}

#[test]
fn conditional_expression() -> Result<()> {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    elaphe::build_from_code_single(&output, "main() { var x = 1; print(x == 2 ? 10 : 20); }")?;
    exec_py_and_assert(&output, "20\n")?;
    elaphe::build_from_code_single(&output, "main() { var x = 2; print(x == 2 ? 10 : 20); }")?;
    exec_py_and_assert(&output, "10\n")?;
    clean(&output);
    Ok(())
}

#[test]
fn logical_expression() -> Result<()> {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    elaphe::build_from_code_single(&output, "main() { var x = 1; print(x == 1 && x == 2); }")?;
    exec_py_and_assert(&output, "False\n")?;
    elaphe::build_from_code_single(&output, "main() { var x = 2; print(x == 1 || x == 2); }")?;
    exec_py_and_assert(&output, "True\n")?;
    elaphe::build_from_code_single(&output, "main() { var x = null; print(x ?? 10); }")?;
    exec_py_and_assert(&output, "10\n")?;
    elaphe::build_from_code_single(&output, "main() { var x = 1; print(x ?? 10); }")?;
    exec_py_and_assert(&output, "1\n")?;
    clean(&output);
    Ok(())
}

#[test]
fn loop_label() -> Result<()> {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
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
    )?;
    exec_py_and_assert(
        &output,
        "0\n0\n1\n2\n3\n10\n0\n1\n2\n3\n20\n30\n0\n1\n2\n3\n40\n",
    )?;
    clean(&output);
    Ok(())
}

#[test]
fn comment() -> Result<()> {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
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
    )?;
    exec_py_and_assert(&output, "1\n")?;
    clean(&output);
    Ok(())
}

#[test]
fn return_value() -> Result<()> {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
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
    )?;
    exec_py_and_assert(&output, "30\n")?;
    clean(&output);
    Ok(())
}

#[test]
fn switch_statement() -> Result<()> {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
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
    )?;
    exec_py_and_assert(&output, "one or two\n")?;
    clean(&output);
    Ok(())
}

#[test]
fn update_expression() -> Result<()> {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    elaphe::build_from_code_single(&output, "main() { var x = 1; print(x++); print(x); }")?;
    exec_py_and_assert(&output, "1\n2\n")?;
    elaphe::build_from_code_single(&output, "main() { var x = 1; print(x--); print(x); }")?;
    exec_py_and_assert(&output, "1\n0\n")?;
    elaphe::build_from_code_single(&output, "main() { var x = 1; print(++x); print(x); }")?;
    exec_py_and_assert(&output, "2\n2\n")?;
    elaphe::build_from_code_single(&output, "main() { var x = 1; print(--x); print(x); }")?;
    exec_py_and_assert(&output, "0\n0\n")?;
    clean(&output);
    Ok(())
}

#[test]
fn try_statement() -> Result<()> {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
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
    )?;
    exec_py_and_assert(&output, "try\nIOError\nfinally\n")?;
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
    )?;
    exec_py_and_assert(&output, "try\nUnknown\nfinally\n")?;
    clean(&output);
    Ok(())
}

#[test]
fn collection_literal() -> Result<()> {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    elaphe::build_from_code_single(&output, "main() { print([1,2,3]); }")?;
    exec_py_and_assert(&output, "[1, 2, 3]\n")?;
    elaphe::build_from_code_single(&output, "main() { print({1,2,3}); }")?;
    exec_py_and_assert(&output, "{1, 2, 3}\n")?;
    elaphe::build_from_code_single(&output, "main() { print({'a':1, 'b':2}); }")?;
    exec_py_and_assert(&output, "{'a': 1, 'b': 2}\n")?;
    clean(&output);
    Ok(())
}

#[test]
fn subscr() -> Result<()> {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    elaphe::build_from_code_single(&output, "main() { var x = [0,1,2]; print(x[2]); }")?;
    exec_py_and_assert(&output, "2\n")?;
    elaphe::build_from_code_single(
        &output,
        "main() { var x = [0,1,2]; print(x[2] = 10); print(x); }",
    )?;
    exec_py_and_assert(&output, "10\n[0, 1, 10]\n")?;
    elaphe::build_from_code_single(
        &output,
        "main() { var x = [0,1,2]; print(x[2] += 10); print(x); }",
    )?;
    exec_py_and_assert(&output, "12\n[0, 1, 12]\n")?;
    elaphe::build_from_code_single(
        &output,
        "main() { var x = [0,1,null]; print(x[2] ??= 10); print(x); }",
    )?;
    exec_py_and_assert(&output, "10\n[0, 1, 10]\n")?;
    elaphe::build_from_code_single(
        &output,
        "main() { var x = [0,1,2]; print(x[2] ??= 10); print(x); }",
    )?;
    exec_py_and_assert(&output, "2\n[0, 1, 2]\n")?;
    clean(&output);
    Ok(())
}

#[test]
fn type_annotation() -> Result<()> {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    elaphe::build_from_code_single(&output, "main() { int x = 1; print(x); }")?;
    exec_py_and_assert(&output, "1\n")?;
    elaphe::build_from_code_single(&output, "main() { final int x = 1; print(x); }")?;
    exec_py_and_assert(&output, "1\n")?;
    elaphe::build_from_code_single(&output, "main() { final x = 1; print(x); }")?;
    exec_py_and_assert(&output, "1\n")?;
    elaphe::build_from_code_single(&output, "main() { const x = 1; print(x); }")?;
    exec_py_and_assert(&output, "1\n")?;
    elaphe::build_from_code_single(&output, "main() { const int x = 1; print(x); }")?;
    exec_py_and_assert(&output, "1\n")?;
    // elaphe::build_from_code_single(&output, "main() { np.int32 x = 1; print(x); }").expect("execution failed.");
    exec_py_and_assert(&output, "1\n")?;
    elaphe::build_from_code_single(&output, "main() { late final int x = 1; print(x); }")?;
    exec_py_and_assert(&output, "1\n")?;
    clean(&output);
    Ok(())
}

#[test]
fn variable_declaration() -> Result<()> {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    elaphe::build_from_code_single(&output, "main() { int x = 1, y = 2; print(x + y); }")?;
    exec_py_and_assert(&output, "3\n")?;
    elaphe::build_from_code_single(&output, "int x = 1, y = 2; main() { print(x + y); }")?;
    exec_py_and_assert(&output, "3\n")?;
    clean(&output);
    Ok(())
}

#[test]
fn function_parameters() -> Result<()> {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
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
    )?;
    exec_py_and_assert(&output, "3\n11\n3\n11\n101\n")?;
    clean(&output);
    Ok(())
}

#[test]
fn class_method() -> Result<()> {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
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
    )?;
    exec_py_and_assert(&output, "Hello!\n")?;
    clean(&output);
    Ok(())
}

#[test]
fn class_field() -> Result<()> {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
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
    )?;
    exec_py_and_assert(&output, "Hello!\n1\n2\n3\n10\n3\n4\n3\n4\n5\n")?;
    clean(&output);
    Ok(())
}

#[test]
fn class_constructor() -> Result<()> {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
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
    )?;
    exec_py_and_assert(&output, "10\n0\n10\n10\n")?;
    clean(&output);
    Ok(())
}

#[test]
fn slice() -> Result<()> {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
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
    )?;
    exec_py_and_assert(&output, "[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]\n[7, 8, 9]\n[3, 4, 5]\n[0, 1, 2]\n[0, 2, 4, 6, 8]\n[3, 6, 9]\n[0, 3]\n[3, 5]\n")?;
    clean(&output);
    Ok(())
}

#[test]
fn type_as_is() -> Result<()> {
    let output = format!("{}.pyc", Uuid::new_v4().hyphenated().to_string());
    elaphe::build_from_code_single(&output, "main() { print(3 is int); }")?;
    exec_py_and_assert(&output, "True\n")?;
    elaphe::build_from_code_single(&output, "main() { print(2 is! int); }")?;
    exec_py_and_assert(&output, "False\n")?;
    elaphe::build_from_code_single(&output, "main() { var x = 1; print(x as int); }")?;
    exec_py_and_assert(&output, "1\n")?;
    elaphe::build_from_code_single(&output, "main() { var as = 1; print(as); }")?;
    exec_py_and_assert(&output, "1\n")?;
    clean(&output);
    Ok(())
}
