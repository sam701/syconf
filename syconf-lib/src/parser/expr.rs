use std::rc::Rc;

use nom::bytes::complete::*;
use nom::sequence::pair;
use nom::{AsChar, IResult, InputLength, InputTake};

use crate::parser::block::BlockExpr;
use crate::parser::comparison::Comparison;
use crate::parser::conditional::Conditional;

use super::*;

#[derive(Debug, Eq, PartialEq)]
pub struct ExprWithLocation<'a> {
    pub inner: Expr<'a>,
    pub location: Span<'a>,
}

// TODO: add special characters in strings, i.e. \n, \t
// TODO: add float type
#[derive(Debug, Eq, PartialEq)]
pub enum Expr<'a> {
    Value(ConfigValue<'a>),
    Block(BlockExpr<'a>),
    Identifier(&'a str),
    FuncDefinition(Rc<FuncDefinition<'a>>),
    Math(Box<MathOperation<'a>>),
    Comparison(Box<Comparison<'a>>),
    Conditional(Box<Conditional<'a>>),
    Logical(Box<Logical<'a>>),
    Suffix(Box<SuffixExpr<'a>>),
    Import(&'a str),
}

impl<'a> Expr<'a> {
    pub fn with_location(self, location: Span<'a>) -> ExprWithLocation<'a> {
        ExprWithLocation {
            inner: self,
            location,
        }
    }
}

pub fn expr(input: Span) -> IResult<Span, ExprWithLocation> {
    expr_logical(input)
}

pub fn identifier(input: Span) -> IResult<Span, &str> {
    let (next_input, (a, b)) = pair(
        take_while1(|x: char| x.is_alpha() || x == '_'),
        take_while(|x: char| x.is_alphanumeric() || x == '_'),
    )(input)?;
    Ok((
        next_input,
        input.take_split(a.input_len() + b.input_len()).1.fragment(),
    ))
}
