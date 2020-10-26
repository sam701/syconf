use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::sequence::{pair, tuple};
use nom::IResult;

use crate::parser::expr::expr;
use crate::parser::{ml_space1, ExprWithLocation, Span};

#[derive(Debug, Eq, PartialEq)]
pub struct Conditional<'a> {
    pub condition: ExprWithLocation<'a>,
    pub then_branch: ExprWithLocation<'a>,
    pub else_branch: ExprWithLocation<'a>,
}

pub fn conditional(input: Span) -> IResult<Span, Conditional> {
    map(
        tuple((
            pair(tag("if"), ml_space1),
            expr,
            tuple((ml_space1, tag("then"), ml_space1)),
            expr,
            tuple((ml_space1, tag("else"), ml_space1)),
            expr,
        )),
        |(_, condition, _, then_branch, _, else_branch)| Conditional {
            condition,
            then_branch,
            else_branch,
        },
    )(input)
}
