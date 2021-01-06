use std::fs::File;
use std::io::Read;
use std::process::Command;

use handlebars::Handlebars;

use crate::compiler::value::FunctionSig;
use crate::compiler::{Error, Value};

pub fn lookup(function_name: &str) -> Option<&'static FunctionSig> {
    Some(match function_name {
        "read_file" => &read_file,
        "getenv" => &getenv,
        "concat" => &concat,
        "shell" => &shell,
        "handlebars" => &handlebars_template,
        _ => return None,
    })
}

fn read_file(args: &[Value]) -> Result<Value, Error> {
    check!(
        args.len() == 1,
        "'read_file' expects a single string argument"
    );
    let file_name = args[0].as_value_string()?;

    let mut buf = String::new();
    let mut f = File::open(file_name.as_ref())
        .map_err(|e| anyhow!("Cannot open file '{}': {}", file_name, e))?;
    f.read_to_string(&mut buf)
        .map_err(|e| anyhow!("Cannot read file '{}': {}", file_name, e))?;
    Ok(Value::String(buf.into()))
}

fn getenv(args: &[Value]) -> Result<Value, Error> {
    check!(
        !args.is_empty() && args.len() <= 2,
        "'getenv' expects a string argument with an optional default value"
    );
    let envname = args[0].as_value_string()?;
    std::env::var(envname.as_ref())
        .map(|x| Value::String(x.into()))
        .or_else(|_| {
            if args.len() == 2 {
                Ok(args[1].clone())
            } else {
                Err(format!("Environment variable '{}' is not set", envname).into())
            }
        })
}

pub fn concat_strings(args: &[Value]) -> Result<Value, Error> {
    let mut out = String::new();
    for s in args {
        match s {
            Value::String(s) => out.push_str(s),
            Value::Number(x) => out.push_str(x.to_string().as_str()),
            Value::Bool(x) => out.push_str(x.to_string().as_str()),
            _ => return Err("Cannot format a non-primitive type".into()),
        }
    }
    Ok(Value::String(out.into()))
}

#[test]
fn func_concat_strings() {
    assert_eq!(
        crate::parse_string(
            r#"
        let name = "mike"
        in
        "Name: ${name}"
    "#
        )
        .unwrap(),
        Value::String("Name: mike".into())
    );
}

fn concat(args: &[Value]) -> Result<Value, Error> {
    check!(
        !args.is_empty(),
        "Concat requires at least one argument as a list"
    );
    let mut out = args[0].as_list()?.to_vec();
    for x in &args[1..] {
        let mut li = x.as_list()?.to_vec();
        out.append(&mut li);
    }
    Ok(Value::List(out.into()))
}

#[test]
fn func_concat() {
    assert_eq!(
        crate::parse_string(r#"concat([1],[2,3],[4]) == [1, 2, 3, 4]"#).unwrap(),
        Value::Bool(true)
    );
}

fn shell(args: &[Value]) -> Result<Value, Error> {
    check!(args.len() == 1, "SHELL requires a single string argument");

    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg(args[0].as_value_string()?.as_ref());
    let out = cmd
        .output()
        .map_err(|e| format!("cannot execute shell script: {}", e))?;
    check!(
        out.status.success(),
        "Command exited with status {}",
        out.status.code().unwrap_or(-1)
    );

    let s =
        String::from_utf8(out.stdout).map_err(|_| "shell script did not return UTF-8 string")?;
    Ok(Value::String(s.into()))
}

#[test]
fn test_shell() {
    assert_eq!(
        crate::parse_string(
            r#"
    shell("echo abc").trim() == "abc"
    "#
        )
        .unwrap(),
        Value::Bool(true)
    );
}

fn handlebars_template(args: &[Value]) -> Result<Value, Error> {
    check!(args.len() == 2, "handlebars function expects two arguments");
    let template = args[0].as_value_string()?;
    Handlebars::new()
        .render_template(template.as_ref(), &args[1].to_serializable())
        .map_err(|e| format!("cannot render template: {}", e).into())
        .map(|x| Value::String(x.into()))
}

#[test]
fn test_template() {
    assert_eq!(
        crate::parse_string(
            r#"
    handlebars("hello {{name}}", {name: "Mouse"}) == "hello Mouse"
    "#
        )
        .unwrap(),
        Value::Bool(true)
    );
}
