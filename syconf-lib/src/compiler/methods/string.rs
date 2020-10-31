use std::cmp::min;
use std::rc::Rc;

use crate::compiler::{Error, Value};

pub type StringMethod = dyn Fn(&str, &[Value]) -> Result<Value, Error>;

pub fn method(method_name: &str) -> Option<&'static StringMethod> {
    Some(match method_name {
        "parse_json" => &parse_json,
        "parse_yaml" => &parse_yaml,
        "parse_toml" => &parse_toml,
        "trim" => &trim,
        "split" => &split,
        "lines" => &lines,
        "unindent" => &unindent,
        _ => return None,
    })
}

fn parse_json(string: &str, args: &[Value]) -> Result<Value, Error> {
    check!(args.is_empty(), "'parse_json' does not take any arguments");
    let x = serde_json::from_str(string).map_err(|e| anyhow!("cannot parse JSON: {}", e))?;
    Ok(Value::HashMap(Rc::new(x)))
}

fn parse_yaml(string: &str, args: &[Value]) -> Result<Value, Error> {
    check!(args.is_empty(), "'parse_yaml' does not take any arguments");
    let x = serde_yaml::from_str(string).map_err(|e| anyhow!("cannot parse YAML: {}", e))?;
    Ok(Value::HashMap(Rc::new(x)))
}

fn parse_toml(string: &str, args: &[Value]) -> Result<Value, Error> {
    check!(args.is_empty(), "'parse_toml' does not take any arguments");
    let x = toml::de::from_str(string).map_err(|e| anyhow!("cannot parse TOML: {}", e))?;
    Ok(Value::HashMap(Rc::new(x)))
}

fn trim(string: &str, args: &[Value]) -> Result<Value, Error> {
    check!(args.is_empty(), "'trim' does not take any arguments");
    Ok(Value::String(string.trim().into()))
}

#[test]
fn trim_string() {
    assert_eq!(
        crate::parse_string(
            r#"
        "
            abc
            ".trim() == "abc"
    "#
        )
        .unwrap(),
        Value::Bool(true)
    )
}

fn split(string: &str, args: &[Value]) -> Result<Value, Error> {
    check!(args.len() == 1, "'split' takes exactly one argument");
    let split_by = args[0].as_value_string()?;
    Ok(Value::List(
        string
            .split(split_by.as_ref())
            .map(|x| Value::String(x.into()))
            .collect(),
    ))
}

#[test]
fn split_string() {
    assert_eq!(
        crate::parse_string(
            r#"
            "aa,bb,cc".split(",") == ["aa", "bb", "cc"]
            "#
        )
        .unwrap(),
        Value::Bool(true)
    )
}

fn unindent(string: &str, args: &[Value]) -> Result<Value, Error> {
    check!(args.is_empty(), "'unindent' does not take any arguments");
    let mut prefixed_whitespaces: Vec<&str> = string
        .lines()
        .filter(|x| !x.trim().is_empty())
        .map(|x| &x[..x.find(|s| !char::is_whitespace(s)).unwrap_or(0)])
        .collect();
    prefixed_whitespaces.sort_unstable();

    let prefix_len = match prefixed_whitespaces.len() {
        0 => return Ok(Value::String(string.into())),
        1 => prefixed_whitespaces[0].len(),
        _ => {
            let first: Vec<char> = prefixed_whitespaces[0].chars().collect();
            let last: Vec<char> = prefixed_whitespaces.last().unwrap().chars().collect();
            let mut cnt = 0;
            for ix in 0..min(first.len(), last.len()) {
                if first[ix] == last[ix] {
                    cnt += 1;
                }
            }
            cnt
        }
    };

    let out = string
        .lines()
        .map(|s| {
            if s.trim().is_empty() {
                ""
            } else {
                &s[prefix_len..]
            }
        })
        .collect::<Vec<&str>>()
        .join("\n");

    Ok(Value::String(out.into()))
}

#[test]
fn func_unindent() {
    assert_eq!(
        crate::parse_string(
            r#"
        "

            abc
        def
                    ghk
        ".unindent()
    "#
        )
        .unwrap(),
        Value::String("\n\n    abc\ndef\n            ghk\n".into())
    )
}

fn lines(string: &str, args: &[Value]) -> Result<Value, Error> {
    check!(args.is_empty(), "'lines' does not expect any argument");
    Ok(Value::List(
        string.lines().map(|x| Value::String(x.into())).collect(),
    ))
}

#[test]
fn test_lines() {
    assert_eq!(
        crate::parse_string(
            r##"
        #"line1
        line2
        line3"#.lines().map((x) => x.trim()) == ["line1", "line2", "line3"]
    "##
        )
        .unwrap(),
        Value::Bool(true)
    )
}
