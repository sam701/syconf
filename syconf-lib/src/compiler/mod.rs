use std::collections::HashMap;

use context::Context;
pub use value::{Func, SerializableValue, TypeMismatch, Value, ValueString};

use crate::parser::ExprWithLocation;
pub use error::{Error, ErrorWithLocation};

#[macro_use]
mod error;
mod compile;
mod context;
mod functions;
mod methods;
mod node;
mod operators;
mod value;
mod value_extraction;

pub fn compile(expr: &ExprWithLocation) -> Result<Value, Error> {
    let node = compile::Compiler.compile(&Context::empty(), expr)?;
    debug!(?node, "compiled node");
    node.resolve(&Context::empty())
}
