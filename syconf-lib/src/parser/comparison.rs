use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::{map, opt, rest_len};
use nom::IResult;
use nom::sequence::{pair, tuple};

use crate::parser::{ConfigValue, Expr, expr_sum, ExprWithLocation, ml_space0};

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


pub fn expr_comparison(input: &str) -> IResult<&str, ExprWithLocation> {
    map(tuple((
        expr_sum,
        opt(pair(
            map(
                tuple((
                    ml_space0,
                    rest_len,
                    alt((
                        map(tag("=="), |_| ComparisonOperator::Equal),
                        map(tag("!="), |_| ComparisonOperator::NotEqual),
                        map(tag(">"), |_| ComparisonOperator::Greater),
                        map(tag("<"), |_| ComparisonOperator::Less),
                        map(tag(">="), |_| ComparisonOperator::GreaterOrEqual),
                        map(tag("<="), |_| ComparisonOperator::LessOrEqual),
                    )),
                    ml_space0,
                )), |(_, rl, op, _)| (rl, op),
            ),
            expr_comparison,
        ))
    )), |(expr1, opt)| match opt {
        Some(((rest_len, operator), expr2)) => Expr::Comparison(Box::new(Comparison {
            expr1,
            expr2,
            operator,
        })).with_location(rest_len),
        None => expr1
    })(input)
}

#[test]
fn test_expr_comparison() {
    assert_eq!(expr_comparison("3 > 2"), Ok(("", Expr::Comparison(Box::new(Comparison {
        expr1: Expr::Value(ConfigValue::Int(3)).with_location(5),
        expr2: Expr::Value(ConfigValue::Int(2)).with_location(1),
        operator: ComparisonOperator::Greater,
    })).with_location(3))))
}