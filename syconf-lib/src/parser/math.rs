use nom::branch::alt;
use nom::bytes::complete::*;
use nom::combinator::{map, opt};
use nom::sequence::{delimited, pair, tuple};
use nom::IResult;

use super::*;
use nom_locate::position;

#[derive(Debug, PartialEq)]
pub struct MathOperation<'a> {
    pub expr1: ExprWithLocation<'a>,
    pub expr2: ExprWithLocation<'a>,
    pub op: MathOp,
}

#[derive(Debug, Eq, PartialEq)]
pub enum MathOp {
    Add,
    Sub,
    Mul,
    Div,
}

pub fn expr_sum(input: Span) -> IResult<Span, ExprWithLocation> {
    map(
        tuple((
            expr_prod,
            opt(pair(
                delimited(
                    ml_space0,
                    map(
                        pair(position, alt((tag("+"), tag("-")))),
                        |(rem, x): (Span, Span)| {
                            (
                                rem,
                                if x.fragment() == &"+" {
                                    MathOp::Add
                                } else {
                                    MathOp::Sub
                                },
                            )
                        },
                    ),
                    ml_space0,
                ),
                expr_sum,
            )),
        )),
        |(a, x)| map_math_op(a, x),
    )(input)
}

pub fn expr_prod(input: Span) -> IResult<Span, ExprWithLocation> {
    map(
        tuple((
            expr_suffix,
            // expr_dot_chain,
            opt(pair(
                delimited(
                    ml_space0,
                    map(
                        pair(position, alt((tag("*"), tag("/")))),
                        |(rem, x): (Span, Span)| {
                            (
                                rem,
                                if x.fragment() == &"*" {
                                    MathOp::Mul
                                } else {
                                    MathOp::Div
                                },
                            )
                        },
                    ),
                    ml_space0,
                ),
                expr_prod,
            )),
        )),
        |(a, x)| map_math_op(a, x),
    )(input)
}

fn map_math_op<'a>(
    expr1: ExprWithLocation<'a>,
    x: Option<((Span<'a>, MathOp), ExprWithLocation<'a>)>,
) -> ExprWithLocation<'a> {
    match x {
        Some(((position, op), expr2)) => {
            Expr::Math(Box::new(MathOperation { expr1, expr2, op })).with_location(position)
        }
        None => expr1,
    }
}
