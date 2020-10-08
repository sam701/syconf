pub use comparison::comparison;
pub use math::math;
pub use logical::*;

use crate::compiler::{Error, Value};

mod math;
mod comparison;
mod logical;

pub fn conditional(args: &[Value]) -> Result<Value, Error> {
    Ok(if args[0].as_bool()? {
        args[1].clone()
    } else {
        args[2].clone()
    })
}