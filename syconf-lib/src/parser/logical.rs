use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::{map, opt};
use nom::sequence::{pair, tuple};
use nom::IResult;

use crate::parser::{expr_comparison, ml_space0, ml_space1, Expr, ExprWithLocation, Span};
use nom_locate::position;

#[derive(Debug, PartialEq)]
pub enum Logical<'a> {
    And(ExprWithLocation<'a>, ExprWithLocation<'a>),
    Or(ExprWithLocation<'a>, ExprWithLocation<'a>),
    Not(ExprWithLocation<'a>),
}

pub fn expr_logical(input: Span) -> IResult<Span, ExprWithLocation> {
    alt((negation, binary))(input)
}

fn binary(input: Span) -> IResult<Span, ExprWithLocation> {
    map(
        tuple((
            expr_comparison,
            opt(pair(
                map(
                    tuple((ml_space0, position, alt((tag("and"), tag("or"))), ml_space0)),
                    |(_, rl, op, _)| (rl, op),
                ),
                expr_logical,
            )),
        )),
        |(expr1, opt)| match opt {
            Some(((pos, operator), expr2)) => {
                let func = if operator.fragment() == &"and" {
                    Logical::And
                } else {
                    Logical::Or
                };
                Expr::Logical(Box::new(func(expr1, expr2))).with_location(pos)
            }
            None => expr1,
        },
    )(input)
}

fn negation(input: Span) -> IResult<Span, ExprWithLocation> {
    map(
        pair(pair(position, pair(tag("not"), ml_space1)), expr_comparison),
        |((pos, _), ex)| Expr::Logical(Box::new(Logical::Not(ex))).with_location(pos),
    )(input)
}
