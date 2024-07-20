#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate derivative;
#[macro_use]
extern crate tracing;

use std::fs::read_to_string;
use std::path::Path;

use parser::*;

pub use crate::resolver::ErrorWithLocation;
pub use crate::resolver::{Func, SerializableValue, TypeMismatch, Value, ValueString};
pub use parser::Number;

mod parser;
mod resolver;

#[cfg(test)]
mod tests;

pub fn parse_string(input: &str) -> Result<Value, ErrorWithLocation> {
    parse_source(Span::new_extra(input, "<input>".into()))
}

pub fn parse_file(file_name: &str) -> Result<Value, ErrorWithLocation> {
    let content = read_to_string(file_name).map_err(|e| ErrorWithLocation {
        location: None,
        message: format!("Cannot read file '{}': {}", file_name, e),
    })?;
    let normalized_fn = std::fs::canonicalize(Path::new(file_name))
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
    parse_source(Span::new_extra(&content, normalized_fn.into()))
}

fn parse_source(source: Span) -> Result<Value, ErrorWithLocation> {
    let (rest, expr) = parse_unit(source)?;
    if !rest.fragment().is_empty() {
        return Err(anyhow!("Cannot parse (incomplete): '{}'", rest.fragment()).into());
    }
    resolver::resolve(&expr)
}
