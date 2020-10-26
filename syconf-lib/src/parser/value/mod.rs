use nom::branch::alt;
use nom::bytes::complete::*;
use nom::character::complete::*;
use nom::combinator::{map, map_res};
use nom::multi::separated_list;
use nom::sequence::{delimited, pair, separated_pair, tuple};
use nom::{IResult, InputLength, InputTake};

use string::ConfigString;

use super::*;
use nom_locate::position;

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

pub fn config_value(input: Span) -> IResult<Span, ConfigValue> {
    alt((
        map(boolean, ConfigValue::Bool),
        map_res(digit1, |s: Span| {
            s.fragment().parse::<i32>().map(ConfigValue::Int)
        }),
        map(hashmap, ConfigValue::HashMap),
        map(list, ConfigValue::List),
        map(string::parse, ConfigValue::String),
    ))(input)
}

fn boolean(input: Span) -> IResult<Span, bool> {
    map(alt((tag("true"), tag("false"))), |x: Span| {
        x.fragment() == &"true"
    })(input)
}

fn hashmap_entry(input: Span) -> IResult<Span, HashMapEntry> {
    map(
        separated_pair(
            alt((
                map(pair(position, identifier), |(rl, id)| {
                    raw_string(id).with_location(rl)
                }),
                expr,
            )),
            tuple((ml_space0, tag(":"), ml_space0)),
            expr,
        ),
        |(key, value)| HashMapEntry { key, value },
    )(input)
}

fn raw_string(s: &str) -> Expr {
    Expr::Value(ConfigValue::String(vec![ConfigString::Raw(s)]))
}

fn sep(input: Span) -> IResult<Span, &str> {
    let orig = input;
    let (input, _) = ml_space0(input)?;
    let (input, _) = tag(",")(input)?;
    let (input, _) = ml_space0(input)?;
    Ok((
        input,
        orig.take_split(orig.input_len() - input.input_len())
            .1
            .fragment(),
    ))
}

fn list(input: Span) -> IResult<Span, Vec<ExprWithLocation>> {
    delimited(
        pair(tag("["), ml_space0),
        separated_list(sep, expr),
        pair(alt((sep, ml_space0)), tag("]")),
    )(input)
}

fn hashmap(input: Span) -> IResult<Span, Vec<HashMapEntry>> {
    delimited(
        pair(tag("{"), ml_space0),
        map(separated_list(alt((sep, ml_space1)), hashmap_entry), |x| {
            x.into_iter().collect()
        }),
        pair(alt((sep, ml_space0)), tag("}")),
    )(input)
}
