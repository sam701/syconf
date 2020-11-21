use crate::ser::to_value;

use std::sync::Arc;
use syconf_lib::Value;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, serde::Serialize)]
struct Abc {
    string: String,
    int: i32,
    boo: bool,
    list: Vec<String>,
    name1: Enum1,
    name2: Enum1,
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, serde::Serialize)]
enum Enum1 {
    Name1(String),
    Name2 { aa: String, bb: i32 },
}

#[test]
fn serialize_struct() {
    let abc = Abc {
        string: "abc".to_owned(),
        boo: true,
        int: 33,
        list: vec!["hello".to_owned()],
        name1: Enum1::Name1("aa".to_owned()),
        name2: Enum1::Name2 {
            aa: "aa".to_owned(),
            bb: 44,
        },
    };

    let val = to_value(&abc).unwrap();
    assert_eq!(
        val,
        Value::HashMap(Arc::new(
            [
                ("string", Value::String("abc".into())),
                ("boo", Value::Bool(true)),
                ("int", Value::Int(33)),
                (
                    "list",
                    Value::List(vec![Value::String("hello".into())].into())
                ),
                ("name1", Value::String("aa".into())),
                (
                    "name2",
                    Value::HashMap(Arc::new(
                        [("aa", Value::String("aa".into())), ("bb", Value::Int(44)),]
                            .iter()
                            .cloned()
                            .map(|(k, v)| (k.into(), v))
                            .collect()
                    ))
                ),
            ]
            .iter()
            .cloned()
            .map(|(k, v)| (k.into(), v))
            .collect()
        ))
    );
}
