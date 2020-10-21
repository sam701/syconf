use crate::compiler::value_extraction::ValueExtractor;
use crate::compiler::{Error, Value};

pub type ListMethod = dyn Fn(&[Value], &[Value]) -> Result<Value, Error>;

pub fn method(name: &str) -> Option<&'static ListMethod> {
    Some(match name {
        "map" => &map,
        "filter" => &filter,
        "len" => &len,
        "append" => &append,
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
            .call(&[Value::Int(ix as i32), val.clone()])?
            .as_bool()?;
        if out {
            filtered.push(val.clone());
        }
    }
    Ok(Value::List(filtered.into()))
}

fn len(list: &[Value], args: &[Value]) -> Result<Value, Error> {
    check!(args.is_empty(), "expects no arguments");
    Ok(Value::Int(list.len() as i32))
}

fn append(list: &[Value], args: &[Value]) -> Result<Value, Error> {
    let mut a = list.to_owned();
    for x in args {
        a.push(x.clone());
    }
    Ok(Value::List(a.into()))
}
