use nom::branch::alt;
use nom::bytes::complete::*;
use nom::combinator::{map, opt, rest_len};
use nom::sequence::{delimited, pair, tuple};
use nom::IResult;

use super::*;

#[derive(Debug, Eq, PartialEq)]
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

pub fn expr_sum(input: &str) -> IResult<&str, ExprWithLocation> {
    map(
        tuple((
            expr_prod,
            opt(pair(
                delimited(
                    ml_space0,
                    map(pair(rest_len, alt((tag("+"), tag("-")))), |(rem, x)| {
                        (rem, if x == "+" { MathOp::Add } else { MathOp::Sub })
                    }),
                    ml_space0,
                ),
                expr_sum,
            )),
        )),
        |(a, x)| map_math_op(a, x),
    )(input)
}

pub fn expr_prod(input: &str) -> IResult<&str, ExprWithLocation> {
    map(
        tuple((
            expr_suffix,
            // expr_dot_chain,
            opt(pair(
                delimited(
                    ml_space0,
                    map(pair(rest_len, alt((tag("*"), tag("/")))), |(rem, x)| {
                        (rem, if x == "*" { MathOp::Mul } else { MathOp::Div })
                    }),
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
    x: Option<((usize, MathOp), ExprWithLocation<'a>)>,
) -> ExprWithLocation<'a> {
    match x {
        Some(((rest_len, op), expr2)) => {
            Expr::Math(Box::new(MathOperation { expr1, expr2, op })).with_location(rest_len)
        }
        None => expr1,
    }
}

#[test]
fn test_math() {
    assert_eq!(
        expr("1 * 2 + 3 * 4"),
        Ok((
            "",
            Expr::Math(Box::new(MathOperation {
                expr1: Expr::Math(Box::new(MathOperation {
                    expr1: Expr::Value(ConfigValue::Int(1)).with_location(13),
                    expr2: Expr::Value(ConfigValue::Int(2)).with_location(9),
                    op: MathOp::Mul,
                }))
                .with_location(11),
                expr2: Expr::Math(Box::new(MathOperation {
                    expr1: Expr::Value(ConfigValue::Int(3)).with_location(5),
                    expr2: Expr::Value(ConfigValue::Int(4)).with_location(1),
                    op: MathOp::Mul,
                }))
                .with_location(3),
                op: MathOp::Add,
            }))
            .with_location(7)
        ))
    );
}
