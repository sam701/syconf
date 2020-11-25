use crate::compiler::Value;
use crate::parse_string;
use std::collections::HashMap;

use crate::parser::number::Number;
use std::sync::Arc;

#[test]
fn error_location() {
    let err = parse_string(
        r#"
    let a = "${x}"
    in
    a
    "#,
    )
    .err()
    .unwrap();
    let line_no = err.location.unwrap().line;
    assert_eq!(line_no, 2);
}

#[test]
fn math() {
    assert_eq!(
        parse_string("1 * 2 + 3 * 4 == 14").unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        parse_string("4.25 + 0.25 == 4.5").unwrap(),
        Value::Bool(true)
    );
}

#[test]
fn interpolated_string() {
    assert_eq!(
        parse_string(
            r#"
    let x = "xx"
    let a = "aa" ;
    let b = 33
    in
    "hello ${x}${ a } ${ b}" == "hello xxaa 33"
    "#
        )
        .unwrap(),
        Value::Bool(true)
    );
}

#[test]
fn empty_string() {
    assert_eq!(
        parse_string(
            r#"
    "" == ''
    "#
        )
        .unwrap(),
        Value::Bool(true)
    );
}

#[test]
fn hashmap() {
    assert_eq!(
        parse_string(
            r#"
    let x = "abc"
    in
    {name: [x]} == {name: ["abc"]}
    "#
        )
        .unwrap(),
        Value::Bool(true)
    );
}

#[test]
fn failure() {
    let err = parse_string(
        r#"
    let a = 3
    abc
    "#,
    )
    .err()
    .unwrap();
    assert_eq!(err.location.unwrap().line, 3);
}

#[test]
fn comparison() {
    // TODO: fix precedence
    assert_eq!(
        parse_string(
            r#"
    (3 > 2) == true
    "#
        )
        .unwrap(),
        Value::Bool(true)
    );
}

#[test]
fn conditional() {
    assert_eq!(
        parse_string(
            r#"
    (if true then 3 else 2) == 3
    "#
        )
        .unwrap(),
        Value::Bool(true)
    );
}

#[test]
fn logical() {
    assert_eq!(
        parse_string(
            r#"
    let a = true
    let b = false
    in
    (a and b) == false
    "#
        )
        .unwrap(),
        Value::Bool(true)
    );
}

#[test]
fn comments() {
    assert_eq!(
        parse_string(
            r#"
    //
    44 == 44 // comment
    "#
        )
        .unwrap(),
        Value::Bool(true)
    );
}

#[test]
fn function_definition() {
    assert_eq!(
        parse_string(
            r#"
    let func = (a, b) => a + b
    in
    func(1,2,) == 3
    "#
        )
        .unwrap(),
        Value::Bool(true)
    );
}

#[test]
fn suffix_operator() {
    assert_eq!(
        parse_string(
            r#"
    let obj = {
        inc: (x) => x + 1
    }
    in
    obj.inc(2) == obj["inc"](2)
    "#
        )
        .unwrap(),
        Value::Bool(true)
    );
}

#[test]
fn plain_config() {
    let mut hm = HashMap::new();
    hm.insert("name".into(), Value::String("winnie the pooh".into()));
    hm.insert("age".into(), Value::Number(Number::Int(3)));
    assert_eq!(
        parse_string(
            r#"
            name: "winnie the pooh"
            age: 3
                  "#
        )
        .unwrap(),
        Value::HashMap(Arc::new(hm))
    );
}

#[test]
fn conditional_evaluation() {
    assert_eq!(
        parse_string(
            r#"
            let ff = (x, acc, f) =>
                if x == 0 then acc else f(x-1, acc+1, f)

            in

            ff(2, 0, ff) == 2
                  "#
        )
        .unwrap(),
        Value::Bool(true)
    );
}

#[test]
fn negative_values() {
    assert_eq!(
        parse_string(
            r#"
            let a = -2
            in
            "${a}" == "-2"
                  "#
        )
        .unwrap(),
        Value::Bool(true)
    );
}
