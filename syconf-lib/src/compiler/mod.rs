use std::collections::HashMap;
use std::rc::Rc;

use context::Context;
pub use source::{Location, Source};
pub use value::Value;

use crate::parser::ExprWithLocation;

mod compile;
mod context;
mod functions;
mod methods;
mod node;
mod operators;
mod source;
mod value;
mod value_extraction;

pub type Error = anyhow::Error;

// TODO: add compiler tests

// TODO: add location to the error messages
pub fn compile(expr: &ExprWithLocation, source: Source) -> Result<Value, Error> {
    let node = compile::Compiler::new(source).compile(&Context::empty(), expr)?;
    debug!(?node, "compiled node");
    node.resolve(&Context::empty())
}
