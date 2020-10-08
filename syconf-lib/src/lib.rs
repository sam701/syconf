#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate derivative;
#[macro_use]
extern crate tracing;

use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use nom::IResult;
use serde::Serialize;

use parser::*;

use crate::compiler::{Error, Source};
pub use crate::compiler::Value;

mod parser;
mod compiler;

pub fn parse_string(input: &str) -> Result<Value, Error> {
    parse_source(Source::from_string(input.to_string()))
}

pub fn parse_file(file_name: &str) -> Result<Value, Error> {
    parse_source(Source::from_file(Path::new(file_name))?)
}

fn parse_source(source: Source) -> Result<Value, Error> {
    let input = source.as_str();
    let (rest, expr) = parse_unit(input).map_err(|e| anyhow!("Cannot parse {}", e))?;
    if !rest.is_empty() {
        bail!("Cannot parse: '{}'", rest);
    }
    compiler::compile(&expr, source.clone())
}

