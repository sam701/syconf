use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::{map, opt};
use nom::multi::many0;
use nom::sequence::{delimited, pair, preceded, terminated, tuple};
use nom::IResult;

use crate::parser::expr::expr;
use crate::parser::leaf::expr_leaf;
use crate::parser::{identifier, ml_space0, Expr, ExprWithLocation, Span};
use nom_locate::position;

#[derive(Debug, PartialEq)]
pub struct SuffixExpr<'a> {
    pub base: ExprWithLocation<'a>,
    pub operator: SuffixOperator<'a>,
}

#[derive(Debug, PartialEq)]
pub enum SuffixOperator<'a> {
    DotField(&'a str),
    FunctionApplication(Vec<ExprWithLocation<'a>>),
    Index(ExprWithLocation<'a>),
}

pub fn expr_suffix(input: Span) -> IResult<Span, ExprWithLocation> {
    map(
        pair(
            expr_leaf,
            many0(pair(preceded(ml_space0, position), suffix_operator)),
        ),
        |(base, list)| {
            list.into_iter().fold(base, |a, (rl, op)| {
                Expr::Suffix(Box::new(SuffixExpr {
                    base: a,
                    operator: op,
                }))
                .with_location(rl)
            })
        },
    )(input)
}

fn suffix_operator(input: Span) -> IResult<Span, SuffixOperator> {
    alt((
        map(dot, SuffixOperator::DotField),
        map(function_application, SuffixOperator::FunctionApplication),
        map(index, SuffixOperator::Index),
    ))(input)
}

fn dot(input: Span) -> IResult<Span, &str> {
    preceded(pair(tag("."), ml_space0), identifier)(input)
}

fn function_application(input: Span) -> IResult<Span, Vec<ExprWithLocation>> {
    delimited(
        pair(tag("("), ml_space0),
        many0(terminated(
            expr,
            opt(tuple((ml_space0, tag(","), ml_space0))),
        )),
        pair(ml_space0, tag(")")),
    )(input)
}

fn index(input: Span) -> IResult<Span, ExprWithLocation> {
    delimited(pair(tag("["), ml_space0), expr, pair(ml_space0, tag("]")))(input)
}
