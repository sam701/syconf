use nom::branch::alt;
use nom::bytes::complete::*;
use nom::combinator::{map, opt};
use nom::sequence::{delimited, pair, tuple};
use nom::IResult;

use super::*;
use nom_locate::position;

#[derive(Debug, PartialEq)]
pub struct BinaryOperatorExpr<'a> {
    pub expr1: ExprWithLocation<'a>,
    pub expr2: ExprWithLocation<'a>,
    pub op: BinaryOperator,
}

#[derive(Debug, Eq, PartialEq)]
pub enum BinaryOperator {
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
                                    BinaryOperator::Add
                                } else {
                                    BinaryOperator::Sub
                                },
                            )
                        },
                    ),
                    ml_space0,
                ),
                cut(expr_sum),
            )),
        )),
        |(a, x)| map_binary_operator(a, x),
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
                                    BinaryOperator::Mul
                                } else {
                                    BinaryOperator::Div
                                },
                            )
                        },
                    ),
                    ml_space0,
                ),
                cut(expr_prod),
            )),
        )),
        |(a, x)| map_binary_operator(a, x),
    )(input)
}

fn map_binary_operator<'a>(
    expr1: ExprWithLocation<'a>,
    x: Option<((Span<'a>, BinaryOperator), ExprWithLocation<'a>)>,
) -> ExprWithLocation<'a> {
    match x {
        Some(((position, op), expr2)) => {
            Expr::BinaryOperator(Box::new(BinaryOperatorExpr { expr1, expr2, op }))
                .with_location(position)
        }
        None => expr1,
    }
}
