use std::collections::HashMap;

use nom::branch::alt;
use nom::bytes::complete::*;
use nom::character::complete::*;
use nom::combinator::{map, map_res, opt};
use nom::IResult;
use nom::multi::separated_list;
use nom::sequence::{delimited, pair, separated_pair, tuple};

use super::*;
use string::ConfigString;

pub mod string;

#[derive(Debug, Eq, PartialEq)]
pub enum ConfigValue<'a> {
    Bool(bool),
    Int(i32),
    String(Vec<ConfigString<'a>>),
    Object(HashMap<&'a str, ExprWithLocation<'a>>),
    List(Vec<ExprWithLocation<'a>>),
}

pub fn config_value(input: &str) -> IResult<&str, ConfigValue> {
    alt((
        map(boolean, ConfigValue::Bool),
        map_res(digit1, |s: &str| s.parse::<i32>().map(ConfigValue::Int)),
        map(object, ConfigValue::Object),
        map(list, ConfigValue::List),
        map(string::parse, |x| ConfigValue::String(x)),
    ))(input)
}

fn boolean(input: &str) -> IResult<&str, bool> {
    map(
        alt((
            tag("true"),
            tag("false"),
        )),
        |x| x == "true",
    )(input)
}

fn object_field(input: &str) -> IResult<&str, (&str, ExprWithLocation)> {
    separated_pair(
        identifier,
        tuple((
            ml_space0,
            tag(":"),
            ml_space0,
        )),
        expr,
    )(input)
}

#[test]
fn test_object_field() {
    assert_eq!((object_field("abc : [ab3]").unwrap().1).1.inner,
               Expr::Value(ConfigValue::List(vec![Expr::Identifier("ab3").with_location(4)]))
    );
}

fn sep(input: &str) -> IResult<&str, &str> {
    let orig = input;
    let (input, _) = ml_space0(input)?;
    let (input, _) = tag(",")(input)?;
    let (input, _) = ml_space0(input)?;
    Ok((input, &orig[..orig.len() - input.len()]))
}

#[test]
fn test_sep() {
    assert_eq!(sep(","), Ok(("", ",")));
    assert_eq!(sep(" , "), Ok(("", " , ")));
}

fn list(input: &str) -> IResult<&str, Vec<ExprWithLocation>> {
    delimited(
        pair(tag("["), ml_space0),
        separated_list(
            sep,
            expr,
        ),
        pair(alt((sep, ml_space0)), tag("]")),
    )(input)
}

#[test]
fn test_list() {
    assert_eq!(list("[abc3]"), Ok(("", vec![
        Expr::Identifier("abc3").with_location(5),
    ])));
    assert_eq!(list("[1, x, 'hello' ]"), Ok(("", vec![
        Expr::Value(ConfigValue::Int(1)).with_location(15),
        Expr::Identifier("x").with_location(12),
        Expr::Value(ConfigValue::String(vec![ConfigString::Raw("hello")])).with_location(9),
    ])));
    assert_eq!(list("[[a]]"), Ok(("", vec![
        Expr::Value(ConfigValue::List(vec![
            Expr::Identifier("a").with_location(3)
        ])).with_location(4),
    ])));
}

fn object(input: &str) -> IResult<&str, HashMap<&str, ExprWithLocation>> {
    delimited(
        pair(tag("{"), ml_space0),
        map(separated_list(
            alt((sep, ml_space1)),
            object_field,
        ), |x| x.into_iter().collect()),
        pair(alt((sep, ml_space0)), tag("}")),
    )(input)
}


#[test]
fn test_object() {
    let mut hm = HashMap::new();
    hm.insert("name", Expr::Value(ConfigValue::String(vec![ConfigString::Raw("earth")])).with_location(50));
    hm.insert("colour", Expr::Identifier("blue").with_location(25));
    hm.insert("li", Expr::Value(ConfigValue::List(vec![Expr::Identifier("ab3").with_location(5)])).with_location(6));
    assert_eq!(object(r#"{
        name: "earth",
        colour: blue ,
        li: [ab3]}"#), Ok(("", hm)));

    let mut hm = HashMap::new();
    hm.insert("name", Expr::Value(ConfigValue::String(vec![ConfigString::Raw("earth")])).with_location(47));
    hm.insert("colour", Expr::Identifier("blue").with_location(23));
    hm.insert("li", Expr::Value(ConfigValue::List(vec![Expr::Identifier("ab3").with_location(5)])).with_location(6));
    assert_eq!(object(r#"{
        name: "earth"
        colour: blue
        li: [ab3]}"#), Ok(("", hm)));
}