use std::rc::Rc;

use nom::branch::alt;
use nom::bytes::complete::*;
use nom::combinator::{map, rest_len};
use nom::IResult;
use nom::sequence::{delimited, pair, tuple};

use crate::parser::block::block_expr;
use crate::parser::conditional::*;

use super::*;

pub fn expr_leaf(input: &str) -> IResult<&str, ExprWithLocation> {
    alt((
        import,
        map(pair(rest_len, conditional), |(rem, x)| Expr::Conditional(Box::new(x)).with_location(rem)),
        map(pair(rest_len, config_value), |(rem, x)| Expr::Value(x).with_location(rem)),
        // map(pair(rest_len, func_call), |(rem, x)| Expr::FuncCall(x).with_location(rem)),
        map(pair(rest_len, identifier), |(rem, x)| Expr::Identifier(x).with_location(rem)),
        map(pair(rest_len, func_definition), |(rem, x)| Expr::FuncDefinition(Rc::new(x)).with_location(rem)),
        delimited(
            pair(tag("("), ml_space0),
            expr,
            pair(ml_space0, tag(")")),
        ),
        map(pair(rest_len, block_expr), |(rem, x)| Expr::Block(x).with_location(rem))
    ))(input)
}

#[test]
fn test_expr_leaf() {
    assert_eq!(expr_leaf("abc"), Ok(((""), Expr::Identifier("abc").with_location(3))));
}

fn import(input: &str) -> IResult<&str, ExprWithLocation> {
    map(
        tuple((
            rest_len,
            tuple((
                tag("import"),
                ml_space1,
                tag("\""),
            )),
            is_not("\""),
            tag("\""),
        )),
        |(rl, _, path, _)| Expr::Import(path).with_location(rl),
    )(input)
}