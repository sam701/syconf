use std::path::PathBuf;
use std::rc::Rc;

use nom::branch::alt;
use nom::combinator::{all_consuming, map};
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
use nom::sequence::delimited;

mod value;
mod math;
mod expr;
mod func;
mod leaf;
mod block;
mod spaces;
mod comparison;
mod conditional;
mod logical;
mod suffix_operators;

pub fn parse_unit(input: &str) -> IResult<&str, ExprWithLocation> {
    alt((
        map(
            all_consuming(block_body),
            |x| Expr::Block(x).with_location(input.len()),
        ),
        delimited( ml_space0, expr, ml_space0)
    ))(input)
}
