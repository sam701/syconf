use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::{all_consuming, cut, map, peek};
use nom::sequence::{delimited, pair, preceded, tuple};
use nom::IResult;

pub use block::{Assignment, BlockExpr};
pub use comparison::*;
pub use conditional::*;
pub use expr::*;
pub use func::*;
pub use logical::*;
pub use math::*;
pub use spaces::*;
pub use suffix_operators::*;
pub use value::*;

use crate::parser::block::block_body;
use std::sync::Arc;

mod block;
mod comparison;
mod conditional;
mod expr;
mod func;
mod leaf;
mod logical;
mod math;
mod spaces;
mod suffix_operators;
#[cfg(test)]
mod test_helpers;
mod value;

pub type Span<'a> = nom_locate::LocatedSpan<&'a str, Arc<str>>;

pub fn parse_unit(input: Span) -> IResult<Span, ExprWithLocation> {
    all_consuming(alt((
        preceded(
            peek(pair(ml_space0, tag("let"))),
            cut(map(block_body, |x| {
                Expr::Block(x).with_location(input.clone())
            })),
        ),
        preceded(
            peek(tuple((ml_space0, identifier, ml_space0, tag(":")))),
            cut(map(hashmap_body, |hm| {
                Expr::Value(ConfigValue::HashMap(hm)).with_location(input.clone())
            })),
        ),
        delimited(ml_space0, expr, ml_space0),
    )))(input.clone())
}
