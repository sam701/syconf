use nom::branch::alt;
use nom::bytes::complete::*;

use nom::combinator::map;

use nom::multi::separated_list0;

use nom::sequence::{delimited, pair, separated_pair, tuple};
use nom::{IResult, InputLength, InputTake};
use nom_locate::position;

pub use number::Number;
use string::ConfigString;

use super::*;

pub mod number;
pub mod string;

#[derive(Debug, PartialEq)]
pub enum ConfigValue<'a> {
    Bool(bool),
    Number(Number),
    String(Vec<ConfigString<'a>>),
    HashMap(Vec<HashMapEntry<'a>>),
    List(Vec<ExprWithLocation<'a>>),
}

#[derive(Debug, PartialEq)]
pub struct HashMapEntry<'a> {
    pub key: ExprWithLocation<'a>,
    pub value: ExprWithLocation<'a>,
}

pub fn config_value(input: Span) -> IResult<Span, ConfigValue> {
    alt((
        map(boolean, ConfigValue::Bool),
        map(number::number, ConfigValue::Number),
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
            cut(expr),
        ),
        |(key, value)| HashMapEntry { key, value },
    )(input)
}

fn raw_string(s: &str) -> Expr {
    Expr::Value(ConfigValue::String(vec![ConfigString::Raw(s)]))
}

fn sep(input: Span) -> IResult<Span, &str> {
    let orig = input.clone();
    let (input, _) = ml_space0(input)?;
    let (input, _) = tag(",")(input)?;
    let (input, _) = ml_space0(input)?;
    Ok((
        input.clone(),
        orig.take_split(orig.input_len() - input.input_len())
            .1
            .fragment(),
    ))
}

fn list(input: Span) -> IResult<Span, Vec<ExprWithLocation>> {
    delimited(
        pair(tag("["), ml_space0),
        separated_list0(sep, expr),
        pair(alt((sep, ml_space0)), tag("]")),
    )(input)
}

fn hashmap(input: Span) -> IResult<Span, Vec<HashMapEntry>> {
    delimited(tag("{"), hashmap_body, tag("}"))(input)
}

pub fn hashmap_body(input: Span) -> IResult<Span, Vec<HashMapEntry>> {
    delimited(
        ml_space0,
        map(separated_list0(alt((sep, ml_space1)), hashmap_entry), |x| {
            x.into_iter().collect()
        }),
        alt((sep, ml_space0)),
    )(input)
}
