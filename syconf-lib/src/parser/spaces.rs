use nom::bytes::complete::*;
use nom::character::complete::{line_ending, not_line_ending};
use nom::combinator::{map, opt, recognize, verify};
use nom::sequence::{pair, preceded, terminated};
use nom::IResult;
use crate::parser::Span;

const SPACES: &str = " \t\r\n";

fn is_space_or_newline(c: char) -> bool {
    SPACES.contains(c)
}

pub fn ml_space1(input: Span) -> IResult<Span, &str> {
    verify(ml_space0, |x: &str| !x.is_empty())(input)
}

#[test]
fn test_ml_space1() {
    use nom::error::ErrorKind;
    assert_eq!(ml_space1(Span::new("  // hello\n")).unwrap().1, "  // hello\n");
    assert_eq!(ml_space1(Span::new("// hello\n//")).unwrap().1, "// hello\n//");
    assert_eq!(
        ml_space1(Span::new("")),
        Err(nom::Err::Error((Span::new(""), ErrorKind::Verify)))
    );
}

pub fn ml_space0(input: Span) -> IResult<Span, &str> {
    map(
        recognize(pair(
            take_while(is_space_or_newline),
            opt(pair(line_comment, ml_space0)),
        )),
        |x| *x.fragment(),
    )(input)
}

#[test]
fn test_ml_space0() {
    assert_eq!(ml_space0(Span::new("  // hello\n")).unwrap().1, "  // hello\n");
    assert_eq!(ml_space0(Span::new("// hello\n//")).unwrap().1, "// hello\n//");
}

fn line_comment(input: Span) -> IResult<Span, &str> {
        map(preceded(
            tag("//"),
            opt(terminated(not_line_ending, line_ending)),
        ),|x| x.map(|a: Span| *a.fragment()).unwrap_or(""))(input)
}

