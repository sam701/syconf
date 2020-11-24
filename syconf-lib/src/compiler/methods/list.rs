use crate::compiler::value::{TypeMismatch, ValueString};
use crate::compiler::value_extraction::ValueExtractor;
use crate::compiler::{Error, Value};

pub type ListMethod = dyn Fn(&[Value], &[Value]) -> Result<Value, Error> + Send + Sync;

pub fn method(name: &str) -> Option<&'static ListMethod> {
    Some(match name {
        "map" => &map,
        "filter" => &filter,
        "len" => &len,
        "append" => &append,
        "join" => &join,
        _ => return None,
    })
}

fn map(list: &[Value], args: &[Value]) -> Result<Value, Error> {
    let func = ValueExtractor::new(args, 1)?.extract_func(0)?;
    let mapped = list
        .iter()
        .map(|x| func.call(&[x.clone()]))
        .collect::<Result<Vec<Value>, Error>>()?;
    Ok(Value::List(mapped.into()))
}

fn filter(list: &[Value], args: &[Value]) -> Result<Value, Error> {
    let func = ValueExtractor::new(args, 1)?.extract_func(0)?;
    let mut filtered = Vec::with_capacity(list.len());
    for (ix, val) in list.iter().enumerate() {
        let out = func
            .call(&[Value::Number(ix.into()), val.clone()])?
            .as_bool()?;
        if out {
            filtered.push(val.clone());
        }
    }
    Ok(Value::List(filtered.into()))
}

fn len(list: &[Value], args: &[Value]) -> Result<Value, Error> {
    check!(args.is_empty(), "expects no arguments");
    Ok(Value::Number(list.len().into()))
}

fn append(list: &[Value], args: &[Value]) -> Result<Value, Error> {
    let mut a = list.to_owned();
    for x in args {
        a.push(x.clone());
    }
    Ok(Value::List(a.into()))
}

fn join(list: &[Value], args: &[Value]) -> Result<Value, Error> {
    check!(args.len() == 1, "'join' takes exactly one argument");
    let strings_to_join = list
        .iter()
        .map(|x| x.as_value_string().map(|x| x.clone()))
        .collect::<Result<Vec<ValueString>, TypeMismatch>>()?;
    let join_by = args[0].as_value_string()?;
    Ok(Value::String(strings_to_join.join(join_by).into()))
}

#[test]
fn join_list_by() {
    assert_eq!(
        crate::parse_string(
            r#"
            ["hello", "world"].join(" ") == "hello world"
            "#
        )
        .unwrap(),
        Value::Bool(true)
    )
}
