use crate::compiler::value::{Func, Method};
use crate::compiler::{Error, Value};

pub mod hashmap;
pub mod list;
pub mod string;

pub fn index(args: &[Value]) -> Result<Value, Error> {
    debug!(?args, "index");
    match &args[0] {
        Value::HashMap(hm) => {
            let key = args[1].as_value_string()?;
            match hm.get(key) {
                Some(v) => Ok(v.clone()),
                None => hashmap::method(key)
                    .map(|func| Value::Func(Func::new_method(Method::HashMap(hm.clone(), func))))
                    .ok_or_else(|| format!("no such field or method: {}", key).into()),
            }
        }
        Value::List(list) => match &args[1] {
            Value::Number(key) => list
                .get(key.as_usize()?)
                .cloned()
                .ok_or_else(|| "No such element".into()),
            Value::String(key) => list::method(key)
                .map(|func| Value::Func(Func::new_method(Method::List(list.clone(), func))))
                .ok_or_else(|| format!("no such field or method: {}", key).into()),
            _ => unreachable!(),
        },
        Value::String(string) => match &args[1] {
            Value::String(method) => string::method(method)
                .map(|func| Value::Func(Func::new_method(Method::String(string.clone(), func))))
                .ok_or_else(|| format!("no such field or method: {}", string).into()),
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}

#[test]
fn method_index() {
    use crate::parse_string;
    assert_eq!(
        parse_string(
            r#"
        {aa:3, bb:4}["aa"] == 3
    "#
        )
        .unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        parse_string(
            r#"
        {aa:3, bb:4}.aa == 3
    "#
        )
        .unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        parse_string(
            r#"
        [1,2,3][1] == 2
    "#
        )
        .unwrap(),
        Value::Bool(true)
    );
}
