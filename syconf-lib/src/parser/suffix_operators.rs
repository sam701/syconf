use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::{map, opt, rest_len};
use nom::multi::many0;
use nom::sequence::{delimited, pair, preceded, terminated, tuple};
use nom::IResult;

use crate::parser::expr::expr;
use crate::parser::leaf::expr_leaf;
use crate::parser::{identifier, ml_space0, Expr, ExprWithLocation};

#[derive(Debug, Eq, PartialEq)]
pub struct SuffixExpr<'a> {
    pub base: ExprWithLocation<'a>,
    pub operator: SuffixOperator<'a>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum SuffixOperator<'a> {
    DotField(&'a str),
    FunctionApplication(Vec<ExprWithLocation<'a>>),
    Index(ExprWithLocation<'a>),
}

pub fn expr_suffix(input: &str) -> IResult<&str, ExprWithLocation> {
    map(
        pair(
            expr_leaf,
            many0(pair(preceded(ml_space0, rest_len), suffix_operator)),
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

#[test]
fn suffix() {
    assert_eq!(
        expr_suffix("a"),
        Ok(("", Expr::Identifier("a").with_location(1)))
    );
    assert_eq!(
        expr_suffix("a.b"),
        Ok((
            "",
            Expr::Suffix(Box::new(SuffixExpr {
                base: Expr::Identifier("a").with_location(3),
                operator: SuffixOperator::DotField("b"),
            }))
            .with_location(2)
        ))
    );
    assert_eq!(
        expr_suffix("a(c)"),
        Ok((
            "",
            Expr::Suffix(Box::new(SuffixExpr {
                base: Expr::Identifier("a").with_location(4),
                operator: SuffixOperator::FunctionApplication(vec![
                    Expr::Identifier("c").with_location(2),
                ]),
            }))
            .with_location(3)
        ))
    );

    assert_eq!(
        expr_suffix("a.b.c"),
        Ok((
            "",
            Expr::Suffix(Box::new(SuffixExpr {
                base: Expr::Suffix(Box::new(SuffixExpr {
                    base: Expr::Identifier("a").with_location(5),
                    operator: SuffixOperator::DotField("b"),
                }))
                .with_location(4),
                operator: SuffixOperator::DotField("c"),
            }))
            .with_location(2)
        ))
    );
}

fn suffix_operator(input: &str) -> IResult<&str, SuffixOperator> {
    alt((
        map(dot, SuffixOperator::DotField),
        map(function_application, SuffixOperator::FunctionApplication),
        map(index, SuffixOperator::Index),
    ))(input)
}

#[test]
fn test_operator() {
    assert_eq!(
        suffix_operator("(c)"),
        Ok((
            "",
            SuffixOperator::FunctionApplication(vec![Expr::Identifier("c").with_location(2),])
        ))
    );
}

fn dot(input: &str) -> IResult<&str, &str> {
    preceded(pair(tag("."), ml_space0), identifier)(input)
}

fn function_application(input: &str) -> IResult<&str, Vec<ExprWithLocation>> {
    delimited(
        pair(tag("("), ml_space0),
        many0(terminated(
            expr,
            opt(tuple((ml_space0, tag(","), ml_space0))),
        )),
        pair(ml_space0, tag(")")),
    )(input)
}

#[test]
fn test_function() {
    assert_eq!(
        function_application("(c, a)"),
        Ok((
            "",
            vec![
                Expr::Identifier("c").with_location(5),
                Expr::Identifier("a").with_location(2),
            ]
        ))
    );
    assert_eq!(
        function_application("(c,)"),
        Ok(("", vec![Expr::Identifier("c").with_location(3),]))
    );
    assert_eq!(function_application("()"), Ok(("", vec![])));
}

fn index(input: &str) -> IResult<&str, ExprWithLocation> {
    delimited(pair(tag("["), ml_space0), expr, pair(ml_space0, tag("]")))(input)
}
