use std::collections::HashMap;

use context::Context;
pub use value::{Func, SerializableValue, TypeMismatch, Value, ValueString};

use crate::parser::ExprWithLocation;
pub use error::{Error, ErrorWithLocation};

#[macro_use]
mod error;
mod context;
mod functions;
mod methods;
mod node;
mod operators;
mod tree_builder;
mod value;
mod value_extraction;

pub fn resolve(expr: &ExprWithLocation) -> Result<Value, Error> {
    let node = tree_builder::NodeTreeBuilder.build_tree(&Context::empty(), expr)?;
    debug!(?node, "compiled node");
    node.resolve(&Context::empty())
}
