use std::cmp::min;
use std::sync::Arc;

use crate::resolver::{Error, Value};

pub type StringMethod = dyn Fn(&str, &[Value]) -> Result<Value, Error> + Send + Sync;

pub fn method(method_name: &str) -> Option<&'static StringMethod> {
    Some(match method_name {
        "parse_json" => &parse_json,
        "parse_yaml" => &parse_yaml,
        "parse_toml" => &parse_toml,
        "trim" => &trim,
        "split" => &split,
        "lines" => &lines,
        "script" => &script,
        "oneline" => &oneline,
        _ => return None,
    })
}

fn parse_json(string: &str, args: &[Value]) -> Result<Value, Error> {
    check!(args.is_empty(), "'parse_json' does not take any arguments");
    let x = serde_json::from_str(string).map_err(|e| anyhow!("cannot parse JSON: {}", e))?;
    Ok(Value::HashMap(Arc::new(x)))
}

fn parse_yaml(string: &str, args: &[Value]) -> Result<Value, Error> {
    check!(args.is_empty(), "'parse_yaml' does not take any arguments");
    let x = serde_yaml::from_str(string).map_err(|e| anyhow!("cannot parse YAML: {}", e))?;
    Ok(Value::HashMap(Arc::new(x)))
}

fn parse_toml(string: &str, args: &[Value]) -> Result<Value, Error> {
    check!(args.is_empty(), "'parse_toml' does not take any arguments");
    let x = toml::de::from_str(string).map_err(|e| anyhow!("cannot parse TOML: {}", e))?;
    Ok(Value::HashMap(Arc::new(x)))
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

#[inline]
fn starts_with_whitespace(s: &str) -> bool {
    s.chars().next().map(|x| x.is_whitespace()).unwrap_or(false)
}

fn unindent(string: &str) -> Vec<&str> {
    let indent_len = string
        .trim_end()
        .lines()
        .enumerate()
        // consider the first line only if it is not empty and starts with spaces
        .filter(|(ix, s)| *ix > 0 || starts_with_whitespace(s))
        .map(|(_, s)| s)
        .filter(|x| !x.trim().is_empty())
        .map(|x| x.find(|s| !char::is_whitespace(s)).unwrap_or(0))
        .min()
        .unwrap_or(0);

    let mut prefix_trimmed = false;
    string
        .trim_end()
        .lines()
        .enumerate()
        .map(|(ix, s)| {
            if ix > 0 || starts_with_whitespace(s) {
                &s[min(s.len(), indent_len)..]
            } else {
                s
            }
        })
        .map(|s| s.trim_end())
        .filter(|s| {
            prefix_trimmed
                || if s.is_empty() {
                    false
                } else {
                    prefix_trimmed = true;
                    true
                }
        })
        .collect()
}

#[test]
fn test_unindent() {
    assert_eq!(
        unindent("aa \n  bb   \n  cc\n\n   "),
        vec!["aa", "bb", "cc"]
    );
    assert_eq!(
        unindent(" aa \n  bb   \n  cc\n\n   "),
        vec!["aa", " bb", " cc"]
    );
    assert_eq!(
        unindent("  aa \n  bb   \n  cc\n\n   "),
        vec!["aa", "bb", "cc"]
    );
    assert_eq!(
        unindent("\n\n  aa \n  bb   \n  cc\n\n   "),
        vec!["aa", "bb", "cc"]
    );
    let empty: Vec<&str> = vec![];
    assert_eq!(unindent(""), empty);
}

fn script(string: &str, args: &[Value]) -> Result<Value, Error> {
    check!(args.is_empty(), "'script' does not take any arguments");
    let out = unindent(string).join("\n");

    Ok(Value::String(out.into()))
}

#[test]
fn string_script() {
    assert_eq!(
        crate::parse_string(
            r#"
            "

            abc
                def
            abc

            ".script() == "abc
    def
abc"
    "#
        )
        .unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        crate::parse_string(
            r#"
            "".script() == ""
    "#
        )
        .unwrap(),
        Value::Bool(true)
    );
}

fn oneline(string: &str, args: &[Value]) -> Result<Value, Error> {
    check!(args.is_empty(), "'oneline' does not take any arguments");
    let out: Vec<&str> = string
        .lines()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    Ok(Value::String(out.join(" ").into()))
}

#[test]
fn string_oneline() {
    assert_eq!(
        crate::parse_string(
            r#"
            "

            abc
                def
            abc

            ".oneline() == "abc def abc"
    "#
        )
        .unwrap(),
        Value::Bool(true)
    );
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
