use nom::branch::alt;
use nom::bytes::complete::{tag, take_while};
use nom::combinator::{all_consuming, map};
use nom::error::ErrorKind;
use nom::multi::many0;
use nom::sequence::{delimited, pair};
use nom::{FindSubstring, IResult, InputLength, InputTake, Needed, Slice};

use crate::parser::expr::expr;
use crate::parser::{ml_space0, ExprWithLocation, Span};

#[derive(Debug, PartialEq)]
pub enum ConfigString<'a> {
    Raw(&'a str),
    Interpolated(ExprWithLocation<'a>),
}

pub fn parse(input: Span) -> IResult<Span, Vec<ConfigString>> {
    let (input, (hashes, quote)) =
        pair(take_while(|x| x == '#'), alt((tag("\""), tag("'"))))(input)?;

    let pattern = format!("{}{}", quote, hashes);
    let input: Span = input;

    match input.fragment().find_substring(&pattern) {
        Some(x) => {
            let out: Span = input.slice(x + pattern.len()..);
            Ok((
                out,
                if quote.fragment() == &"'" {
                    vec![ConfigString::Raw(&input.fragment()[..x])]
                } else {
                    all_consuming(many0(interpolated_string))(input.slice(..x))?.1
                },
            ))
        }
        None => Err(nom::Err::Incomplete(Needed::Unknown)),
    }
}

#[test]
fn raw_string_vec() {
    use crate::parser::test_helpers::span;

    assert_eq!(
        parse(span("\"hello\n\"")).unwrap().1,
        vec![ConfigString::Raw("hello\n")]
    );
    assert_eq!(
        parse(span(r#"'hello'"#)).unwrap().1,
        vec![ConfigString::Raw("hello")]
    );
    assert_eq!(
        parse(span(r##"#"abco""#"##)).unwrap().1,
        vec![ConfigString::Raw("abco\"")]
    );
}

fn interpolated_string(input: Span) -> IResult<Span, ConfigString> {
    if input.fragment().is_empty() {
        return Err(nom::Err::Error(nom::error::Error {
            input,
            code: ErrorKind::Eof,
        }));
    }
    match input.fragment().find_substring("${") {
        Some(x) if x == 0 => map(
            delimited(ml_space0, expr, pair(ml_space0, tag("}"))),
            ConfigString::Interpolated,
        )(input.take_split(2).0),
        Some(x) => {
            let (rest, _res) = input.take_split(x);
            Ok((rest, ConfigString::Raw(&input.fragment()[..x])))
        }
        None => Ok((
            input.slice(input.input_len()..),
            ConfigString::Raw(input.fragment()),
        )),
    }
}
