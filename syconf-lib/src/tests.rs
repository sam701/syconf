use crate::compiler::Value;
use crate::parse_string;

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
    assert_eq!(dbg!(err).location.unwrap().line, 3);
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
