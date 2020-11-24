use crate::compiler::value::FunctionSig;
use crate::compiler::{Error, Value};
use crate::parser::number::Number;
use crate::parser::MathOp;
use std::ops::{Add, Div, Mul, Sub};

pub fn math(op: &MathOp) -> &'static FunctionSig {
    match op {
        MathOp::Add => &op_add,
        MathOp::Sub => &op_sub,
        MathOp::Mul => &op_mul,
        MathOp::Div => &op_div,
    }
}

macro_rules! bin_op {
    ($func:ident, $op:ident) => {
        fn $func(args: &[Value]) -> Result<Value, Error> {
            check!(args.len() == 2, "expects 2 arguments");
            match (&args[0], &args[1]) {
                (Value::Number(Number::Int(x)), Value::Number(Number::Int(y))) => {
                    Ok(Value::Number(Number::Int(x.$op(y))))
                }
                (Value::Number(Number::Float(x)), Value::Number(Number::Float(y))) => {
                    Ok(Value::Number(Number::Float(x.$op(y))))
                }
                _ => Err("expects either INT and INT or FLOAT and FLOAT".into()),
            }
        }
    };
}

bin_op!(op_add, add);
bin_op!(op_sub, sub);
bin_op!(op_mul, mul);
bin_op!(op_div, div);

// fn op_add(oargs: &[Value]) -> Result<Value, Error> {
//     math_bi_op(|a, b| a + b, oargs)
//     // bin_op!(a, b, a + b)
// }
// fn op_sub(oargs: &[Value]) -> Result<Value, Error> {
//     math_bi_op(|a, b| a - b, oargs)
// }
// fn op_mul(oargs: &[Value]) -> Result<Value, Error> {
//     math_bi_op(|a, b| a * b, oargs)
// }
// fn op_div(oargs: &[Value]) -> Result<Value, Error> {
//     math_bi_op(|a, b| a / b, oargs)
// }
//
// fn math_bi_op<F: Fn(&Number, &Number) -> Number>(f: F, args: &[Value]) -> Result<Value, Error> {
//     check!(args.len() == 2, "expects 2 arguments");
//     match (&args[0], &args[1]) {
//         (Value::Number(a), Value::Number(b)) => Ok(Value::Number(f(a, b))),
//         _ => Err(format!(
//             "Expects NUMBER and NUMBER, but was {:?} and {:?}",
//             &args[0], &args[1]
//         )
//         .into()),
//     }
// }
