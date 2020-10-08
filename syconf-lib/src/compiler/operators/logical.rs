use crate::compiler::{Value, Error};

pub fn and(args: &[Value]) -> Result<Value, Error> {
    Ok(Value::Bool(args[0].as_bool()? && args[1].as_bool()?))
}

pub fn or(args: &[Value]) -> Result<Value, Error> {
    Ok(Value::Bool(args[0].as_bool()? || args[1].as_bool()?))
}

pub fn not(args: &[Value]) -> Result<Value, Error> {
    Ok(Value::Bool(!args[0].as_bool()?))
}

