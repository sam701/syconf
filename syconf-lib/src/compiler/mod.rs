use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use nom::lib::std::fmt::Formatter;

use context::Context;
pub use source::{Location, Source};
pub use value::Value;

use crate::parser::ExprWithLocation;

mod source;
mod context;
mod value;
mod node;
mod compile;
mod methods;
mod value_extraction;
mod operators;
mod functions;


pub type Error = anyhow::Error;

// TODO: add compiler tests

// TODO: add location to the error messages
pub fn compile(expr: &ExprWithLocation, source: Source) -> Result<Value, Error> {
    let node = compile::Compiler::new(source).compile(&Context::empty(), expr)?;
    debug!(?node, "compiled node");
    node.resolve(&mut Context::empty())
}



