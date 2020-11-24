use nom::bytes::complete::*;
use nom::combinator::{map, opt};
use nom::multi::separated_nonempty_list;
use nom::sequence::{delimited, pair, tuple};
use nom::IResult;

use super::*;

#[derive(Debug, PartialEq)]
pub struct Assignment<'a>(pub &'a str, pub ExprWithLocation<'a>);

fn assignment(input: Span) -> IResult<Span, Assignment> {
    map(
        tuple((
            pair(tag("let"), ml_space1),
            identifier,
            tuple((ml_space0, tag("="), ml_space0)),
            expr,
            opt(pair(ml_space0, tag(";"))),
        )),
        |(_, ident, _, ex, _)| Assignment(ident, ex),
    )(input)
}

#[derive(Debug, PartialEq)]
pub struct BlockExpr<'a> {
    pub local_assignments: Vec<Assignment<'a>>,
    pub expression: Box<ExprWithLocation<'a>>,
}

pub fn block_body(input: Span) -> IResult<Span, BlockExpr> {
    map(
        delimited(
            ml_space0,
            pair(
                pair(
                    separated_nonempty_list(ml_space1, assignment),
                    tuple((ml_space1, tag("in"), ml_space1)),
                ),
                expr,
            ),
            ml_space0,
        ),
        |(la, expression)| BlockExpr {
            local_assignments: la.0,
            expression: Box::new(expression),
        },
    )(input)
}

pub fn block_expr(input: Span) -> IResult<Span, BlockExpr> {
    delimited(tag("{"), block_body, tag("}"))(input)
}
