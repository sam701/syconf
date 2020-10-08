use nom::{FindSubstring, IResult, Needed};
use nom::branch::alt;
use nom::bytes::complete::{tag, take_till, take_until, take_while};
use nom::combinator::{all_consuming, map, not, opt};
use nom::error::ErrorKind;
use nom::multi::{many0, many1};
use nom::sequence::{delimited, pair, preceded, terminated};

use crate::parser::{Expr, ExprWithLocation, ml_space0};
use crate::parser::expr::expr;

#[derive(Debug, Eq, PartialEq)]
pub enum ConfigString<'a> {
    Raw(&'a str),
    Interpolated(ExprWithLocation<'a>),
}


pub fn parse(input: &str) -> IResult<&str, Vec<ConfigString>> {
    let (input, (hashes, quote)) = pair(
        take_while(|x| x == '#'),
        alt((
            tag("\""),
            tag("'"),
        )),
    )(input)?;

    let pattern = format!("{}{}", quote, hashes);

    match input.find_substring(&pattern) {
        Some(x) => Ok((
            &input[x + pattern.len()..],
            if quote == "'" {
                vec![ConfigString::Raw(&input[..x])]
            } else {
                all_consuming(many1(interpolated_string))(&input[..x])?.1
            }
        )),
        None => Err(nom::Err::Incomplete(Needed::Unknown))
    }
}

#[test]
fn raw_string_vec() {
    assert_eq!(parse("\"hello\n\""), Ok(("", vec![ConfigString::Raw("hello\n")])));
    assert_eq!(parse(r#"'hello'"#), Ok(("", vec![ConfigString::Raw("hello")])));
    assert_eq!(parse(r##"#"abco""#"##), Ok(("", vec![ConfigString::Raw("abco\"")])));
}

#[test]
fn interpolated_string_vec() {
    assert_eq!(parse("\"hello${x}${a} ${b} \""), Ok(("", vec![
        ConfigString::Raw("hello"),
        ConfigString::Interpolated(Expr::Identifier("x").with_location(12)),
        ConfigString::Interpolated(Expr::Identifier("a").with_location(8)),
        ConfigString::Raw(" "),
        ConfigString::Interpolated(Expr::Identifier("b").with_location(3)),
        ConfigString::Raw(" "),
    ])));
}


// TODO: add offset to locations
fn interpolated_string(input: &str) -> IResult<&str, ConfigString> {
    if input.is_empty() {
        return Err(nom::Err::Error((input, ErrorKind::Eof)));
    }
    match input.find_substring("${") {
        Some(x) if x == 0 =>
            map(
                delimited(ml_space0, expr, pair(ml_space0, tag("}"))),
                ConfigString::Interpolated,
            )(&input[2..]),
        Some(x) => Ok((&input[x..], ConfigString::Raw(&input[..x]))),
        None => Ok(("", ConfigString::Raw(input))),
    }
}

#[test]
fn test_interpolated_string() {
    assert_eq!(interpolated_string("abc"), Ok(("", ConfigString::Raw("abc"))));
    assert_eq!(interpolated_string("abc ${x}"), Ok(("${x}", ConfigString::Raw("abc "))));
    assert_eq!(interpolated_string("${ x }"), Ok(("", ConfigString::Interpolated(
        Expr::Identifier("x").with_location(3)
    ))));
}