use std::fs::File;
use std::io::Read;
use std::process::Command;
use std::sync::Arc;

use crate::compiler::value::FunctionSig;
use crate::compiler::{Error, Value};

pub fn lookup(function_name: &str) -> Option<&'static FunctionSig> {
    Some(match function_name {
        "read_file" => &read_file,
        "getenv" => &getenv,
        "concat" => &concat,
        "merge" => &merge,
        "fold" => &fold,
        "shell" => &shell,
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

fn merge(args: &[Value]) -> Result<Value, Error> {
    check!(
        !args.is_empty(),
        "Merge requires at least one argument as a hashmap or a list of hashmaps"
    );
    let hm_list = if let Value::List(list) = &args[0] {
        check!(
            args.len() == 1,
            "Merge expects either multiple hashmaps or a single list of hashmaps"
        );
        list.as_ref()
    } else {
        args
    };
    let mut out = hm_list[0].as_hashmap()?.clone();
    for x in &hm_list[1..] {
        let li = x.as_hashmap()?.clone();
        out.extend(li.into_iter());
    }
    Ok(Value::HashMap(Arc::new(out)))
}

#[test]
fn func_merge() {
    use crate::Number;

    let mut hm = std::collections::HashMap::new();
    hm.insert("name".into(), Value::String("alexei".into()));
    hm.insert("age".into(), Value::Number(Number::Int(40)));
    assert_eq!(
        crate::parse_string(
            r#"merge(
        {name: "john"},
        {name: "alexei"},
        {age: 40},
    )"#
        )
        .unwrap(),
        Value::HashMap(Arc::new(hm))
    );

    assert_eq!(
        crate::parse_string(
            r#"merge([
        {name: "john"},
        {age: 40},
    ]) == {name: "john", age: 40}"#
        )
        .unwrap(),
        Value::Bool(true)
    );
}

fn fold(args: &[Value]) -> Result<Value, Error> {
    check!(
        args.len() == 3,
        "Fold requires 3 arguments (initial value, accumulation function, list or hashmap)"
    );
    let func = args[1].as_func()?;
    match &args[2] {
        Value::List(list) => {
            let mut out = args[0].clone();
            for (ix, val) in list.iter().enumerate() {
                let args = &[out.clone(), Value::Number(ix.into()), val.clone()];
                out = func.call(args)?;
            }
            Ok(out)
        }
        Value::HashMap(hm) => {
            let mut out = args[0].clone();
            for (ix, val) in hm.iter() {
                let args = &[out.clone(), Value::String(ix.clone()), val.clone()];
                out = func.call(args)?;
            }
            Ok(out)
        }
        _ => Err("3rd argument must be either a list or a hashmap".into()),
    }
}

#[test]
fn func_fold() {
    use crate::Number;

    assert_eq!(
        crate::parse_string(r#"fold(0, (acc, ix, val) => acc + val, [1,2,3])"#).unwrap(),
        Value::Number(Number::Int(6))
    );
    assert_eq!(
        crate::parse_string(
            r#"fold(0, (acc, ix, val) => acc + val, {
        aa: 1,
        bb: 2,
        cc: 3
    })"#
        )
        .unwrap(),
        Value::Number(Number::Int(6))
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
