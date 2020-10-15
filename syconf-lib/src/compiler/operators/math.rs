use crate::compiler::{Error, Value};
use crate::parser::MathOp;

pub fn math(op: &MathOp) -> &'static dyn Fn(&[Value]) -> Result<Value, Error> {
    match op {
        MathOp::Add => &op_add,
        MathOp::Sub => &op_sub,
        MathOp::Mul => &op_mul,
        MathOp::Div => &op_div,
    }
}

fn op_add(oargs: &[Value]) -> Result<Value, Error> {
    math_bi_op(|a, b| a + b, oargs)
}
fn op_sub(oargs: &[Value]) -> Result<Value, Error> {
    math_bi_op(|a, b| a - b, oargs)
}
fn op_mul(oargs: &[Value]) -> Result<Value, Error> {
    math_bi_op(|a, b| a * b, oargs)
}
fn op_div(oargs: &[Value]) -> Result<Value, Error> {
    math_bi_op(|a, b| a / b, oargs)
}

fn math_bi_op<F: Fn(i32, i32) -> i32>(f: F, args: &[Value]) -> Result<Value, Error> {
    ensure!(args.len() == 2, "expects 2 arguments");
    match (&args[0], &args[1]) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(f(*a, *b))),
        _ => bail!(
            "Expects INT and INT, but was {:?} and {:?}",
            &args[0],
            &args[1]
        ),
    }
}
