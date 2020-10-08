use nom::bytes::complete::*;
use nom::combinator::{map, opt};
use nom::IResult;
use nom::multi::separated_list;
use nom::sequence::{delimited, pair, separated_pair, tuple};

use super::*;

#[derive(Debug, Eq, PartialEq)]
pub struct FuncDefinition<'a> {
    pub arguments: Vec<&'a str>,
    pub expression: Box<ExprWithLocation<'a>>,
}


fn func_arguments(input: &str) -> IResult<&str, Vec<&str>> {
    delimited(
        pair(tag("("), ml_space0),
        separated_list(
            tuple((ml_space0, tag(","), ml_space0)),
            identifier,
        ),
        pair(ml_space0, tag(")")),
    )(input)
}

pub fn func_definition(input: &str) -> IResult<&str, FuncDefinition> {
    map(
        separated_pair(
            func_arguments,
            tuple((ml_space0, tag("=>"), ml_space0)),
            expr,
        ),
        |(arguments, ex)| FuncDefinition {
            arguments,
            expression: Box::new(ex),
        },
    )(input)
}

#[test]
fn test_func_definition() {
    assert_eq!(func_definition("(a, b ) => a + b"), Ok(("", FuncDefinition {
        arguments: vec!["a", "b"],
        expression: Box::new(Expr::Math(Box::new(MathOperation {
            expr1: Expr::Identifier("a").with_location(5),
            expr2: Expr::Identifier("b").with_location(1),
            op: MathOp::Add,
        })).with_location(3)),
    })));
}