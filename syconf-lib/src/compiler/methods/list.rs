use std::rc::Rc;

use crate::compiler::{Error, Value};
use crate::compiler::context::Context;
use crate::compiler::value_extraction::ValueExtractor;

pub fn method(name: &str) -> Option<&'static dyn Fn(&Vec<Value>, &[Value]) -> Result<Value, Error>> {
    Some(match name {
        "get" => &get,
        "map" => &map,
        "filter" => &filter,
        "len" => &len,
        "append" => &append,
        _ => return None
    })
}

fn get(list: &Vec<Value>, args: &[Value]) -> Result<Value, Error> {
    let key = ValueExtractor::new(args, 1)?.extract_int(0)?;
    list.get(key as usize).map(Clone::clone).ok_or(anyhow!("list of length {} does not have index '{}'", list.len(), key))
}

fn map(list: &Vec<Value>, args: &[Value]) -> Result<Value, Error> {
    let func = ValueExtractor::new(args, 1)?.extract_func(0)?;
    let mapped = list.iter().map(|x| func.call(&[x.clone()])).collect::<Result<Vec<Value>, Error>>()?;
    Ok(Value::List(Rc::new(mapped)))
}

fn filter(list: &Vec<Value>, args: &[Value]) -> Result<Value, Error> {
    let func = ValueExtractor::new(args, 1)?.extract_func(0)?;
    let mut filtered = Vec::with_capacity(list.len());
    for (ix, val) in list.iter().enumerate() {
        let out = func.call(&[Value::Int(ix as i32), val.clone()])?.as_bool()?;
        if out {
            filtered.push(val.clone());
        }
    }
    Ok(Value::List(Rc::new(filtered)))
}

fn len(list: &Vec<Value>, args: &[Value]) -> Result<Value, Error> {
    ensure!(args.len() == 0, "expects no arguments");
    Ok(Value::Int(list.len() as i32))
}

fn append(list: &Vec<Value>, args: &[Value]) -> Result<Value, Error> {
    let mut a = list.clone();
    for x in args {
        a.push(x.clone());
    }
    Ok(Value::List(Rc::new(a)))
}
