use std::collections::HashMap;
use std::rc::Rc;

use context::Context;
pub use source::{Location, Source};
pub use value::Value;

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
mod source;
mod value;
mod value_extraction;

pub fn compile(expr: &ExprWithLocation, source: Source) -> Result<Value, Error> {
    let node = compile::Compiler::new(source).compile(&Context::empty(), expr)?;
    debug!(?node, "compiled node");
    node.resolve(&Context::empty())
}
