use nom::bytes::complete::*;
use nom::combinator::map;
use nom::multi::separated_list0;
use nom::sequence::{delimited, pair, separated_pair, tuple};
use nom::IResult;

use super::*;

#[derive(Debug, PartialEq)]
pub struct FuncDefinition<'a> {
    pub arguments: Vec<&'a str>,
    pub expression: Box<ExprWithLocation<'a>>,
}

fn func_arguments(input: Span) -> IResult<Span, Vec<&str>> {
    delimited(
        pair(tag("("), ml_space0),
        separated_list0(tuple((ml_space0, tag(","), ml_space0)), identifier),
        pair(ml_space0, tag(")")),
    )(input)
}

pub fn func_definition(input: Span) -> IResult<Span, FuncDefinition> {
    map(
        separated_pair(
            func_arguments,
            tuple((ml_space0, tag("=>"), ml_space0)),
            cut(expr),
        ),
        |(arguments, ex)| FuncDefinition {
            arguments,
            expression: Box::new(ex),
        },
    )(input)
}
