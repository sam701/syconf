use crate::compiler::{Error, Value};
use crate::parser::ComparisonOperator;

pub fn comparison(op: &ComparisonOperator) -> &'static dyn Fn(&[Value]) -> Result<Value, Error> {
    use ComparisonOperator::*;
    match op {
        Equal => &equal,
        NotEqual => &not_equal,
        Greater => &greater,
        GreaterOrEqual => &greater_or_equal,
        Less => &less,
        LessOrEqual => &less_or_equal,
    }
}

fn equal(args: &[Value]) -> Result<Value, Error> {
    Ok(Value::Bool(args[0] == args[1]))
}

#[test]
fn op_equal() {
    use crate::parse_string;
    assert_eq!(
        parse_string(
            r#"
        3 == 3
    "#
        )
        .unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        parse_string(
            r#"
        {aa: 33} == {aa:33}
    "#
        )
        .unwrap(),
        Value::Bool(true)
    );
}

fn not_equal(args: &[Value]) -> Result<Value, Error> {
    Ok(Value::Bool(args[0] != args[1]))
}

fn greater(args: &[Value]) -> Result<Value, Error> {
    Ok(Value::Bool(args[0] > args[1]))
}

fn greater_or_equal(args: &[Value]) -> Result<Value, Error> {
    Ok(Value::Bool(args[0] >= args[1]))
}

fn less(args: &[Value]) -> Result<Value, Error> {
    Ok(Value::Bool(args[0] < args[1]))
}

fn less_or_equal(args: &[Value]) -> Result<Value, Error> {
    Ok(Value::Bool(args[0] <= args[1]))
}
