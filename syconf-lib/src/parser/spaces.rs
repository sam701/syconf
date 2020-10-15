use nom::bytes::complete::*;
use nom::character::complete::{line_ending, not_line_ending};
use nom::combinator::{map, opt, recognize, verify};

use nom::sequence::{pair, preceded, terminated};
use nom::IResult;

const SPACES: &str = " \t\r\n";

fn is_space_or_newline(c: char) -> bool {
    SPACES.contains(c)
}

pub fn ml_space1(input: &str) -> IResult<&str, &str> {
    verify(ml_space0, |x: &str| !x.is_empty())(input)
}

#[test]
fn test_ml_space1() {
    use nom::error::ErrorKind;
    assert_eq!(ml_space1("  // hello\n"), Ok(("", "  // hello\n")));
    assert_eq!(ml_space1("// hello\n//"), Ok(("", "// hello\n//")));
    assert_eq!(ml_space1(""), Err(nom::Err::Error(("", ErrorKind::Verify))));
}

pub fn ml_space0(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        take_while(is_space_or_newline),
        opt(pair(line_comment, ml_space0)),
    ))(input)
}

#[test]
fn test_ml_space0() {
    assert_eq!(ml_space0("  // hello\n"), Ok(("", "  // hello\n")));
    assert_eq!(ml_space0("// hello\n//"), Ok(("", "// hello\n//")));
}

fn line_comment(input: &str) -> IResult<&str, &str> {
    preceded(
        tag("//"),
        map(opt(terminated(not_line_ending, line_ending)), |x| {
            x.unwrap_or("")
        }),
    )(input)
}

#[test]
fn test_line_comment() {
    assert_eq!(line_comment("// hello\nabc"), Ok(("abc", " hello")));
    assert_eq!(line_comment("//"), Ok(("", "")));
}
