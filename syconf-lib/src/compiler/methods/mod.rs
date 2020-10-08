use crate::compiler::{Error, Value};
use crate::compiler::value::{Func, Method};
use crate::parse_string;

pub mod hashmap;
pub mod list;
pub mod string;


pub fn index(args: &[Value]) -> Result<Value, Error> {
    debug!(?args, "index");
    match &args[0] {
        Value::HashMap(hm) => {
            let key = args[1].as_str()?;
            match hm.get(key) {
                Some(v) => Ok(v.clone()),
                None => hashmap::method(key)
                    .map(|func| Value::Func(Func::new_method(Method::HashMap(hm.clone(), func))))
                    .ok_or(anyhow!("no such field or method: {}", key))
            }
        }
        Value::List(list) => match &args[1] {
            Value::Int(key) =>
                list.get(*key as usize).map(Clone::clone).ok_or(anyhow!("No such element")),
            Value::String(key) =>
                list::method(key)
                    .map(|func| Value::Func(Func::new_method(Method::List(list.clone(), func))))
                    .ok_or(anyhow!("no such field or method: {}", key)),
            _ => unreachable!()
        },
        Value::String(string) => match &args[1] {
            Value::String(method) => string::method(method)
                .map(|func| Value::Func(Func::new_method(Method::String(string.clone(), func))))
                .ok_or(anyhow!("no such field or method: {}", string)),
            _ => unreachable!()
        }
        _ => unreachable!()
    }
}

#[test]
fn method_index() {
    assert_eq!(parse_string(r#"
        {aa:3, bb:4}["aa"] == 3
    "#).unwrap(), Value::Bool(true));
    assert_eq!(parse_string(r#"
        {aa:3, bb:4}.aa == 3
    "#).unwrap(), Value::Bool(true));
    assert_eq!(parse_string(r#"
        [1,2,3][1] == 2
    "#).unwrap(), Value::Bool(true));
}