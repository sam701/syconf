use std::rc::Rc;

use nom::branch::alt;
use nom::bytes::complete::*;
use nom::combinator::map;
use nom::sequence::{delimited, pair, tuple};
use nom::IResult;

use crate::parser::block::block_expr;
use crate::parser::conditional::*;

use super::*;
use nom_locate::position;

pub fn expr_leaf(input: Span) -> IResult<Span, ExprWithLocation> {
    alt((
        import,
        map(pair(position, conditional), |(pos, x)| {
            Expr::Conditional(Box::new(x)).with_location(pos)
        }),
        map(pair(position, config_value), |(pos, x)| {
            Expr::Value(x).with_location(pos)
        }),
        // map(pair(position, func_call), |(pos, x)| Expr::FuncCall(x).with_location(pos)),
        map(pair(position, identifier), |(pos, x)| {
            Expr::Identifier(x).with_location(pos)
        }),
        map(pair(position, func_definition), |(pos, x)| {
            Expr::FuncDefinition(Rc::new(x)).with_location(pos)
        }),
        delimited(pair(tag("("), ml_space0), expr, pair(ml_space0, tag(")"))),
        map(pair(position, block_expr), |(pos, x)| {
            Expr::Block(x).with_location(pos)
        }),
    ))(input)
}

fn import(input: Span) -> IResult<Span, ExprWithLocation> {
    map(
        tuple((
            position,
            tuple((tag("import"), ml_space1, tag("\""))),
            is_not("\""),
            tag("\""),
        )),
        |(pos, _, path, _)| Expr::Import(path.fragment()).with_location(pos),
    )(input)
}
