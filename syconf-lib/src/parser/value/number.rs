use std::fmt;

use nom::combinator::map;

use nom::number::complete::double;

use nom::IResult;

use serde::{Deserialize, Serialize};

use super::*;
use crate::resolver::Error;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Number {
    Int(i64),
    Float(f64),
}

impl Number {
    pub fn as_usize(&self) -> Result<usize, Error> {
        match self {
            Number::Int(x) => Ok(*x as usize),
            _ => Err("expects int".into()),
        }
    }
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Number::Int(x) => write!(f, "{}", x),
            Number::Float(x) => write!(f, "{}", x),
        }
    }
}

impl From<usize> for Number {
    fn from(x: usize) -> Self {
        Number::Int(x as i64)
    }
}

pub fn number(input: Span) -> IResult<Span, Number> {
    map(double, |x| {
        if x.fract() == 0.0 {
            Number::Int(x as i64)
        } else {
            Number::Float(x)
        }
    })(input)
}
