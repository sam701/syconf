use super::{Value, Error};
use super::value::Func;
use nom::lib::std::collections::HashMap;
use std::rc::Rc;

pub struct ValueExtractor<'a>(&'a [Value]);

impl<'a> ValueExtractor<'a> {
    pub fn new(args: &'a [Value], required_arg_count: usize) -> Result<Self, Error> {
        ensure!(args.len() == required_arg_count, "expects {} arguments", required_arg_count);
        Ok(Self(args))
    }

    pub fn extract_string(&self, ix: usize) -> Result<&str, Error> {
        if let Value::String(s) = &self.0[ix] {
            Ok(s.as_str())
        } else {
            Err(anyhow!("expects a string"))
        }
    }

    pub fn extract_int(&self, ix: usize) -> Result<i32, Error> {
        if let Value::Int(x) = &self.0[ix] {
            Ok(*x)
        } else {
            Err(anyhow!("expects an int"))
        }
    }

    pub fn extract_bool(&self, ix: usize) -> Result<bool, Error> {
        if let Value::Bool(x) = &self.0[ix] {
            Ok(*x)
        } else {
            Err(anyhow!("expects a boolean"))
        }
    }

    pub fn extract_list(&self, ix: usize) -> Result<&[Value], Error> {
        if let Value::List(x) = &self.0[ix] {
            Ok(x.as_slice())
        } else {
            Err(anyhow!("expects a boolean"))
        }
    }

    pub fn extract_hashmap(&self, ix: usize) -> Result<&HashMap<String, Value>, Error> {
        if let Value::HashMap(x) = &self.0[ix] {
            Ok(x)
        } else {
            Err(anyhow!("expects a hashmap"))
        }
    }

    pub fn extract_func(&self, ix: usize) -> Result<Func, Error> {
        if let Value::Func(x) = &self.0[ix] {
            Ok(x.clone())
        } else {
            Err(anyhow!("expects a function"))
        }
    }
}

impl Value {

}