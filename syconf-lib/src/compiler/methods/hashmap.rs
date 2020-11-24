use std::collections::HashMap;

use crate::compiler::value::ValueString;
use crate::compiler::value_extraction::ValueExtractor;
use crate::compiler::{Error, Value};
use std::sync::Arc;

pub type HashmapMethod =
    dyn Fn(&HashMap<ValueString, Value>, &[Value]) -> Result<Value, Error> + Send + Sync;

pub fn method(name: &str) -> Option<&'static HashmapMethod> {
    Some(match name {
        "map" => &map,
        "filter" => &filter,
        "len" => &len,
        "insert" => &insert,
        _ => return None,
    })
}

fn map(hm: &HashMap<ValueString, Value>, args: &[Value]) -> Result<Value, Error> {
    let extractor = ValueExtractor::new(args, 1)?;
    let func = extractor.extract_func(0)?;

    let mut new_hm = HashMap::new();
    for (k, v) in hm {
        let v = func.call(&[Value::String(k.clone()), v.clone()])?;
        match v {
            Value::List(list) => {
                let ex = ValueExtractor::new(list.as_ref(), 2)?;
                new_hm.insert(ex.extract_string(0)?.clone(), list[1].clone());
            }
            _ => return Err("hashmap map function must return a list of 2 values".into()),
        }
    }
    Ok(Value::HashMap(Arc::new(new_hm)))
}

#[test]
fn func_map() {
    assert_eq!(
        crate::parse_string(
            r#"
        {aa:3, bb:4}.map((k,v) => [k, v * 10]) == {bb: 40, aa: 30}
    "#
        )
        .unwrap(),
        Value::Bool(true)
    )
}

fn filter(hm: &HashMap<ValueString, Value>, args: &[Value]) -> Result<Value, Error> {
    let func = ValueExtractor::new(args, 1)?.extract_func(0)?;
    let mut filtered = HashMap::with_capacity(hm.len());
    for (ix, val) in hm {
        let out = func
            .call(&[Value::String(ix.clone()), val.clone()])?
            .as_bool()?;
        if out {
            filtered.insert(ix.clone(), val.clone());
        }
    }
    Ok(Value::HashMap(Arc::new(filtered)))
}

#[test]
fn func_filter() {
    assert_eq!(
        crate::parse_string(
            r#"
        {aa:3, bb:4}.filter((k,v) => k == "bb") == {bb: 4}
    "#
        )
        .unwrap(),
        Value::Bool(true)
    )
}

fn len(hm: &HashMap<ValueString, Value>, args: &[Value]) -> Result<Value, Error> {
    check!(args.is_empty(), "expects no arguments");
    Ok(Value::Number(hm.len().into()))
}

#[test]
fn func_len() {
    assert_eq!(
        crate::parse_string(
            r#"
        {aa:3, bb:4}.len() == 2
    "#
        )
        .unwrap(),
        Value::Bool(true)
    )
}

fn insert(hm: &HashMap<ValueString, Value>, args: &[Value]) -> Result<Value, Error> {
    check!(args.len() == 2, "expects 2 arguments");
    let mut out = hm.clone();
    out.insert(args[0].as_value_string()?.clone(), args[1].clone());
    Ok(Value::HashMap(Arc::new(out)))
}

#[test]
fn func_insert() {
    assert_eq!(
        crate::parse_string(
            r#"
        {aa: 33}.insert("bb", "abc") == {aa:33, bb:"abc"}
    "#
        )
        .unwrap(),
        Value::Bool(true)
    )
}

#[test]
fn key_expr() {
    assert_eq!(
        crate::parse_string(
            r#"
        let x = 3
        in
        {"abc${x}": 33} == {abc3:33}
    "#
        )
        .unwrap(),
        Value::Bool(true)
    )
}
