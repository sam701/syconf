use nom::bytes::complete::*;
use nom::combinator::{map, opt};
use nom::multi::separated_nonempty_list;
use nom::sequence::{delimited, pair, tuple};
use nom::IResult;

use super::*;

#[derive(Debug, Eq, PartialEq)]
pub struct Assignment<'a>(pub &'a str, pub ExprWithLocation<'a>);

fn assignment(input: &str) -> IResult<&str, Assignment> {
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

#[test]
fn test_assignment() {
    assert_eq!(
        assignment("let _ab3 =  44"),
        Ok((
            "",
            Assignment("_ab3", Expr::Value(ConfigValue::Int(44)).with_location(2),)
        ))
    );

    assert_eq!(
        assignment("let _ab3 =  44 ;"),
        Ok((
            "",
            Assignment("_ab3", Expr::Value(ConfigValue::Int(44)).with_location(4),)
        ))
    );
}

#[derive(Debug, Eq, PartialEq)]
pub struct BlockExpr<'a> {
    pub local_assignments: Vec<Assignment<'a>>,
    pub expression: Box<ExprWithLocation<'a>>,
}

pub fn block_body(input: &str) -> IResult<&str, BlockExpr> {
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

#[test]
fn failed_block() {
    use nom::error::ErrorKind;

    let a = pair(
        pair(
            separated_nonempty_list(ml_space1, assignment),
            tuple((ml_space1, tag("in"), ml_space1)),
        ),
        expr,
    )("let a = 3 abc");
    assert_eq!(a, Err(nom::Err::Error(("abc", ErrorKind::Tag))));
}

#[test]
fn test_block_body() {
    assert_eq!(
        block_body(
            r#"
        let x = 33
        let b = x + 4
        in
        b
    "#
        ),
        Ok((
            "",
            BlockExpr {
                local_assignments: vec![
                    Assignment("x", Expr::Value(ConfigValue::Int(33)).with_location(50)),
                    Assignment(
                        "b",
                        Expr::Math(Box::new(MathOperation {
                            expr1: Expr::Identifier("x").with_location(31),
                            expr2: Expr::Value(ConfigValue::Int(4)).with_location(27),
                            op: MathOp::Add,
                        }))
                        .with_location(29)
                    ),
                ],
                expression: Box::new(Expr::Identifier("b").with_location(6)),
            }
        ))
    );
}

pub fn block_expr(input: &str) -> IResult<&str, BlockExpr> {
    delimited(tag("{"), block_body, tag("}"))(input)
}
