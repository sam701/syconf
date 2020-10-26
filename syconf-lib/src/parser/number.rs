use nom::branch::alt;
use nom::bytes::complete::*;
use nom::character::complete::*;
use nom::combinator::{map, map_res, opt};
use nom::IResult;
use nom::multi::separated_nonempty_list;
use nom::sequence::{delimited, pair, tuple, preceded};

#[derive(Debug, Eq, PartialEq)]
pub enum Number {
    Int(i64),
    Float(f64),
}

pub fn number(input: Input) -> IResult<Input, Number> {
    map(
        tuple((
            opt(alt((tag("-"), tag("+")))),
            digit1,
            opt(preceded(
                tag("."),
                digit0,
            ))
        )),
        |(sign, d1, d2)| {
            let sign = if sign.unwrap_or("+") == '+' {1} else {-1};
            if let Some(frac) = d2 {

            }
        },
    )(input)
}
