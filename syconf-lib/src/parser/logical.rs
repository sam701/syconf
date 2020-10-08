use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::{map, opt, rest_len};
use nom::IResult;
use nom::sequence::{pair, separated_pair, tuple};

use crate::parser::{ConfigValue, Expr, expr_comparison, expr_sum, ExprWithLocation, ml_space0, ml_space1};

#[derive(Debug, Eq, PartialEq)]
pub enum Logical<'a> {
    And(ExprWithLocation<'a>, ExprWithLocation<'a>),
    Or(ExprWithLocation<'a>, ExprWithLocation<'a>),
    Not(ExprWithLocation<'a>),
}


pub fn expr_logical(input: &str) -> IResult<&str, ExprWithLocation> {
    alt((
        negation,
        binary
    ))(input)
}

fn binary(input: &str) -> IResult<&str, ExprWithLocation> {
    map(tuple((
        expr_comparison,
        opt(pair(
            map(
                tuple((
                    ml_space0,
                    rest_len,
                    alt((
                        tag("and"),
                        tag("or"),
                    )),
                    ml_space0,
                )), |(_, rl, op, _)| (rl, op),
            ),
            expr_logical,
        ))
    )), |(expr1, opt)| match opt {
        Some(((rest_len, operator), expr2)) => {
            let func = if operator == "and" { Logical::And } else { Logical::Or };
            Expr::Logical(Box::new(
                func(expr1, expr2)
            )).with_location(rest_len)
        }
        None => expr1
    })(input)
}

fn negation(input: &str) -> IResult<&str, ExprWithLocation> {
    map(
        pair(
            pair(rest_len,
                 pair(tag("not"), ml_space1),
            ),
            expr_comparison,
        ), |((rl, _), ex)| Expr::Logical(Box::new(Logical::Not(ex))).with_location(rl),
    )(input)
}

#[test]
fn test_expr_logical() {
    assert_eq!(expr_logical("a and b"), Ok(("", Expr::Logical(Box::new(Logical::And(
        Expr::Identifier("a").with_location(7),
        Expr::Identifier("b").with_location(1),
    ))).with_location(5))));
}