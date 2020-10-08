use std::rc::Rc;

use nom::{AsChar, IResult};
use nom::bytes::complete::*;
use nom::sequence::pair;

use crate::parser::block::BlockExpr;
use crate::parser::comparison::{Comparison, expr_comparison};
use crate::parser::conditional::Conditional;

use super::*;

#[derive(Debug, Eq, PartialEq)]
pub struct ExprWithLocation<'a> {
    pub inner: Expr<'a>,
    pub rest_len: usize,
}

// TODO: add multiline string
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
    pub fn with_location(self, rest_len: usize) -> ExprWithLocation<'a> {
        ExprWithLocation {
            inner: self,
            rest_len,
        }
    }
}

pub fn expr(input: &str) -> IResult<&str, ExprWithLocation> {
    expr_logical(input)
}

pub fn identifier(input: &str) -> IResult<&str, &str> {
    let (next_input, (a, b)) = pair(
        take_while1(|x: char| x.is_alpha() || x == '_'),
        take_while(|x: char| x.is_alphanumeric() || x == '_'))(input)?;
    Ok((next_input, &input[..a.len() + b.len()]))
}

#[test]
fn test_identifier() {
    assert_eq!(identifier("abc"), Ok(("", "abc")));
    assert_ne!(identifier("3ab"), Ok(("", "3a")));
}
