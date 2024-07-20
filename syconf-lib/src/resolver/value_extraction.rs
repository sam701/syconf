use super::value::Func;
use super::{Error, Value};
use crate::resolver::value::ValueString;

pub struct ValueExtractor<'a>(&'a [Value]);

impl<'a> ValueExtractor<'a> {
    pub fn new(args: &'a [Value], required_arg_count: usize) -> Result<Self, Error> {
        check!(
            args.len() == required_arg_count,
            "expects {} arguments",
            required_arg_count
        );
        Ok(Self(args))
    }

    pub fn extract_string(&self, ix: usize) -> Result<&ValueString, Error> {
        if let Value::String(s) = &self.0[ix] {
            Ok(s)
        } else {
            Err("expects a string".into())
        }
    }

    pub fn extract_func(&self, ix: usize) -> Result<Func, Error> {
        if let Value::Func(x) = &self.0[ix] {
            Ok(x.clone())
        } else {
            Err("expects a function".into())
        }
    }
}
