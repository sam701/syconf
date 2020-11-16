use std::collections::HashMap;

use serde::Deserialize;

use crate::{from_str, Function};

#[derive(Deserialize, Eq, PartialEq, Debug)]
struct Abc {
    name: String,
    age: i32,
    cool: bool,
    nicknames: Vec<String>,
    labels: HashMap<String, String>,
    enum1: UntaggedEnum,
    tup: (String, i32),
    ch: char,
    newtype: Newtype,
    option_none: Option<String>,
    option_some: Option<String>,
}

#[derive(Deserialize, Eq, PartialEq, Debug)]
struct Newtype(String);

#[derive(Deserialize, Eq, PartialEq, Debug)]
#[serde(untagged)]
enum UntaggedEnum {
    String1(String),
    String2 { content: String },
}

#[derive(Deserialize, Eq, PartialEq, Debug)]
#[serde(tag = "type", content = "abc")]
enum InternallyTaggedEnum {
    String0,
    String1(String),
    String2 { content: String },
}

#[derive(Deserialize, Eq, PartialEq, Debug)]
enum ExternallyTaggedEnum {
    String0,
    String1(String),
    String2 { content: String },
    String3(String, i32),
}

#[test]
fn deserialize_struct() {
    let abc: Abc = from_str(
        r#"
        let name = "pooh"
        in
        {
            name: name
            age: 3
            cool: true
            nicknames: ['winnie']
            labels: {
                street: "tree"
            }
            enum1: "aa"
            tup: ["bb", 33]
            ch: "a"
            newtype: "type"
            option_some: "bb"
        }
    "#,
    )
    .unwrap();
    let mut labels = HashMap::new();
    labels.insert("street".to_owned(), "tree".to_owned());
    assert_eq!(
        abc,
        Abc {
            name: "pooh".to_owned(),
            age: 3,
            cool: true,
            nicknames: vec!["winnie".to_owned()],
            labels,
            enum1: UntaggedEnum::String1("aa".to_owned()),
            tup: ("bb".to_owned(), 33),
            ch: 'a',
            newtype: Newtype("type".to_owned()),
            option_none: None,
            option_some: Some("bb".to_owned()),
        }
    )
}

#[test]
fn deserialize_untagged_enum() {
    let abc: UntaggedEnum = from_str(r#" "aa" "#).unwrap();
    assert_eq!(abc, UntaggedEnum::String1("aa".to_owned()));

    let abc: UntaggedEnum = from_str(r#" {content: "aa"} "#).unwrap();
    assert_eq!(
        abc,
        UntaggedEnum::String2 {
            content: "aa".to_owned()
        }
    );
}

#[test]
fn deserialize_internally_tagged_enum() {
    let abc: InternallyTaggedEnum = from_str(r#" {type: "String0"} "#).unwrap();
    assert_eq!(abc, InternallyTaggedEnum::String0);

    let abc: InternallyTaggedEnum = from_str(r#" {type: "String1", abc: "aa"}"#).unwrap();
    assert_eq!(abc, InternallyTaggedEnum::String1("aa".to_owned()));

    let abc: InternallyTaggedEnum =
        from_str(r#" {type: "String2", abc: {content: "aa"}}"#).unwrap();
    assert_eq!(
        abc,
        InternallyTaggedEnum::String2 {
            content: "aa".to_owned()
        }
    );
}

#[test]
fn deserialize_externally_tagged_enum() {
    let abc: ExternallyTaggedEnum = from_str(r#" {String0: {}}"#).unwrap();
    assert_eq!(abc, ExternallyTaggedEnum::String0);

    let abc: ExternallyTaggedEnum = from_str(r#" {String1: "aa"}"#).unwrap();
    assert_eq!(abc, ExternallyTaggedEnum::String1("aa".to_owned()));

    let abc: ExternallyTaggedEnum = from_str(r#" {String2: {content: "aa"}}"#).unwrap();
    assert_eq!(
        abc,
        ExternallyTaggedEnum::String2 {
            content: "aa".to_owned()
        }
    );

    let abc: ExternallyTaggedEnum = from_str(r#" {String3: ["aa", 33]}"#).unwrap();
    assert_eq!(abc, ExternallyTaggedEnum::String3("aa".to_owned(), 33));
}

#[test]
fn deserialize_functions() {
    #[derive(Debug, Deserialize)]
    struct Abc {
        func: Function,
    }

    #[derive(Debug, serde::Serialize)]
    struct Input {
        number: i32,
    }

    let abc: Abc = from_str(
        r#"
        func: (x) => x.number + 2
    "#,
    )
    .unwrap();

    let result: i32 = abc.func.call1(&Input { number: 3 }).unwrap();

    assert_eq!(result, 5);
}
