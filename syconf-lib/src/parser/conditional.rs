use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::{map, opt, rest_len};
use nom::IResult;
use nom::sequence::{pair, tuple};

use crate::parser::{Expr, ExprWithLocation, ml_space1, ConfigValue};
use crate::parser::expr::expr;

#[derive(Debug, Eq, PartialEq)]
pub struct Conditional<'a> {
    pub condition: ExprWithLocation<'a>,
    pub then_branch: ExprWithLocation<'a>,
    pub else_branch: ExprWithLocation<'a>,
}

pub fn conditional(input: &str) -> IResult<&str, Conditional> {
    map(tuple((
        rest_len,
        pair(
            tag("if"),
            ml_space1,
        ),
        expr,
        tuple((
            ml_space1,
            tag("then"),
            ml_space1,
        )),
        expr,
        tuple((
            ml_space1,
            tag("else"),
            ml_space1,
        )),
        expr,
    )), |(rl, _, condition, _, then_branch, _, else_branch)|
            Conditional {
                condition,
                then_branch,
                else_branch,
            })(input)
}

#[test]
fn test_conditional() {
    assert_eq!(conditional("if true then 3 else 2"), Ok(("", Conditional {
        condition: Expr::Value(ConfigValue::Bool(true)).with_location(18),
        then_branch: Expr::Value(ConfigValue::Int(3)).with_location(8),
        else_branch: Expr::Value(ConfigValue::Int(2)).with_location(1),
    })))
}