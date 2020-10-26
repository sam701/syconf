use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::{map, opt};
use nom::sequence::{pair, tuple};
use nom::IResult;

use crate::parser::{expr_sum, ml_space0, Expr, ExprWithLocation, Span};
use nom_locate::position;

#[derive(Debug, Eq, PartialEq)]
pub struct Comparison<'a> {
    pub expr1: ExprWithLocation<'a>,
    pub expr2: ExprWithLocation<'a>,
    pub operator: ComparisonOperator,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    Less,
    Greater,
    LessOrEqual,
    GreaterOrEqual,
}

pub fn expr_comparison(input: Span) -> IResult<Span, ExprWithLocation> {
    map(
        tuple((
            expr_sum,
            opt(pair(
                map(
                    tuple((
                        ml_space0,
                        position,
                        alt((
                            map(tag("=="), |_| ComparisonOperator::Equal),
                            map(tag("!="), |_| ComparisonOperator::NotEqual),
                            map(tag(">"), |_| ComparisonOperator::Greater),
                            map(tag("<"), |_| ComparisonOperator::Less),
                            map(tag(">="), |_| ComparisonOperator::GreaterOrEqual),
                            map(tag("<="), |_| ComparisonOperator::LessOrEqual),
                        )),
                        ml_space0,
                    )),
                    |(_, rl, op, _)| (rl, op),
                ),
                expr_comparison,
            )),
        )),
        |(expr1, opt)| match opt {
            Some(((pos, operator), expr2)) => Expr::Comparison(Box::new(Comparison {
                expr1,
                expr2,
                operator,
            }))
            .with_location(pos),
            None => expr1,
        },
    )(input)
}
