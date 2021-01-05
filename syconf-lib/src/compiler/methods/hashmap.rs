use std::collections::HashMap;
use std::sync::Arc;

use crate::compiler::value::ValueString;
use crate::compiler::value_extraction::ValueExtractor;
use crate::compiler::{Error, Value};

pub type HashmapMethod =
    dyn Fn(&HashMap<ValueString, Value>, &[Value]) -> Result<Value, Error> + Send + Sync;

pub fn method(name: &str) -> Option<&'static HashmapMethod> {
    Some(match name {
        "map" => &map,
        "filter" => &filter,
        "len" => &len,
        "insert" => &insert,
        "merge" => &merge,
        "drop" => &drop,
        "to_list" => &to_list,
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

fn merge(hm: &HashMap<ValueString, Value>, args: &[Value]) -> Result<Value, Error> {
    check!(args.len() == 1, "expects one hashmap as argument");
    let mut out = hm.clone();
    let other = args[0].as_hashmap()?;
    out = merge_raw(out, other);

    Ok(Value::HashMap(Arc::new(out)))
}

fn merge_raw(
    mut dest: HashMap<ValueString, Value>,
    src: &HashMap<ValueString, Value>,
) -> HashMap<ValueString, Value> {
    for (k, v) in src {
        let fv = match (dest.get(k), v) {
            (Some(Value::HashMap(hm1)), Value::HashMap(hm2)) => {
                Value::HashMap(Arc::new(merge_raw(hm1.as_ref().clone(), hm2.as_ref())))
            }
            _ => v.clone(),
        };
        dest.insert(k.clone(), fv);
    }
    dest
}

#[test]
fn test_merge() {
    assert_eq!(
        crate::parse_string(
            r#"
        {
            a: {
                b: {
                    c: 10
                }
            }
            n1: 3
            n2: 4
        }.merge({
            a: {
                b: {
                    d: 20
                }
            }
            n2: 5
        }) == {
            a: {
                b: {
                    c: 10
                    d: 20
                }
            }
            n1: 3
            n2: 5
        }
    "#
        )
        .unwrap(),
        Value::Bool(true)
    )
}

fn drop(hm: &HashMap<ValueString, Value>, args: &[Value]) -> Result<Value, Error> {
    check!(args.len() == 1, "expects one string argument");
    let mut out = hm.clone();
    let path: Vec<&str> = args[0].as_value_string()?.split('.').collect();
    out = drop_raw(out, path.as_slice())?;

    Ok(Value::HashMap(Arc::new(out)))
}

fn drop_raw(
    mut hm: HashMap<ValueString, Value>,
    path: &[&str],
) -> Result<HashMap<ValueString, Value>, Error> {
    match path.len() {
        0 => {}
        1 => {
            hm.remove(path[0]);
        }
        _ => {
            let key = path[0];
            match hm.get(key) {
                Some(Value::HashMap(hm2)) => {
                    let cloned = hm2.as_ref().clone();
                    hm.insert(
                        key.into(),
                        Value::HashMap(Arc::new(drop_raw(cloned, &path[1..])?)),
                    );
                }
                Some(_) => {
                    return Err(
                        "Cannot drop hashmap key, because the object is not a hashmap".into(),
                    );
                }
                None => {}
            }
        }
    }
    Ok(hm)
}

#[test]
fn test_drop() {
    assert_eq!(
        crate::parse_string(
            r#"
        {
            a: {
                b: {
                    c: 10
                    d: 20
                }
            }
        }.drop("a.b.d") == {
            a: {
                b: {
                    c: 10
                }
            }
        }
    "#
        )
        .unwrap(),
        Value::Bool(true)
    )
}

fn to_list(hm: &HashMap<ValueString, Value>, args: &[Value]) -> Result<Value, Error> {
    check!(args.is_empty(), "to_list does not take any arguments");
    let mut list: Vec<Value> = hm
        .iter()
        .map(|(k, v)| Value::List(vec![Value::String(k.clone()), v.clone()].into()))
        .collect();
    list.sort_by_key(|x| {
        x.as_list().expect("list")[0]
            .as_value_string()
            .expect("string")
            .clone()
    });
    Ok(Value::List(list.into()))
}

#[test]
fn test_to_list() {
    assert_eq!(
        crate::parse_string(
            r#"
        {aa: 3, bb: 4}.to_list() == [
            ["aa", 3],
            ["bb", 4],
        ]
    "#
        )
        .unwrap(),
        Value::Bool(true)
    )
}
