use std::collections::HashMap;

use nom::branch::alt;
use nom::bytes::complete::*;
use nom::character::complete::*;
use nom::combinator::{map, map_res, opt, rest_len};
use nom::IResult;
use nom::multi::separated_list;
use nom::sequence::{delimited, pair, separated_pair, tuple};

use string::ConfigString;

use super::*;

pub mod string;

#[derive(Debug, Eq, PartialEq)]
pub enum ConfigValue<'a> {
    Bool(bool),
    Int(i32),
    String(Vec<ConfigString<'a>>),
    HashMap(Vec<HashMapEntry<'a>>),
    List(Vec<ExprWithLocation<'a>>),
}

#[derive(Debug, Eq, PartialEq)]
pub struct HashMapEntry<'a> {
    pub key: ExprWithLocation<'a>,
    pub value: ExprWithLocation<'a>,
}

pub fn config_value(input: &str) -> IResult<&str, ConfigValue> {
    alt((
        map(boolean, ConfigValue::Bool),
        map_res(digit1, |s: &str| s.parse::<i32>().map(ConfigValue::Int)),
        map(hashmap, ConfigValue::HashMap),
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

fn hashmap_entry(input: &str) -> IResult<&str, HashMapEntry> {
    map(
        separated_pair(
            alt((
                map(pair(
                    rest_len,
                    identifier,
                ), |(rl, id)| raw_string(id).with_location(rl)),
                expr,
            )),
            tuple((
                ml_space0,
                tag(":"),
                ml_space0,
            )),
            expr,
        ), |(key, value)| HashMapEntry { key, value },
    )(input)
}

fn raw_string(s: &str) -> Expr {
    Expr::Value(ConfigValue::String(vec![ConfigString::Raw(s)]))
}

#[test]
fn test_hashmap_entry() {
    assert_eq!(hashmap_entry("abc : [ab3]"),
               Ok(("", HashMapEntry {
                   key: raw_string("abc").with_location(11),
                   value: Expr::Value(ConfigValue::List(vec![
                       Expr::Identifier("ab3").with_location(4)
                   ])).with_location(5),
               }))
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

fn hashmap(input: &str) -> IResult<&str, Vec<HashMapEntry>> {
    delimited(
        pair(tag("{"), ml_space0),
        map(separated_list(
            alt((sep, ml_space1)),
            hashmap_entry,
        ), |x| x.into_iter().collect()),
        pair(alt((sep, ml_space0)), tag("}")),
    )(input)
}


#[test]
fn test_object() {
    assert_eq!(hashmap(r#"{
        name: "earth"
        colour: blue ,
        li: [ab3]}"#), Ok(("", vec![
        HashMapEntry{
            key: raw_string("name").with_location(55),
            value: Expr::Value(ConfigValue::String(vec![ConfigString::Raw("earth")])).with_location(49),
        },
        HashMapEntry{
            key: raw_string("colour").with_location(33),
            value: Expr::Identifier("blue").with_location(25),
        },
        HashMapEntry{
            key: raw_string("li").with_location(10),
            value: Expr::Value(ConfigValue::List(vec![Expr::Identifier("ab3").with_location(5)])).with_location(6),
        },
    ])));
}