use super::value::Func;
use super::{Error, Value};

pub struct ValueExtractor<'a>(&'a [Value]);

impl<'a> ValueExtractor<'a> {
    pub fn new(args: &'a [Value], required_arg_count: usize) -> Result<Self, Error> {
        ensure!(
            args.len() == required_arg_count,
            "expects {} arguments",
            required_arg_count
        );
        Ok(Self(args))
    }

    pub fn extract_string(&self, ix: usize) -> Result<&str, Error> {
        if let Value::String(s) = &self.0[ix] {
            Ok(s)
        } else {
            Err(anyhow!("expects a string"))
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
