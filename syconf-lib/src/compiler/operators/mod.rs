pub use comparison::comparison;
pub use logical::*;
pub use math::math;

use crate::compiler::{Error, Value};

mod comparison;
mod logical;
mod math;

pub fn conditional(args: &[Value]) -> Result<Value, Error> {
    Ok(if args[0].as_bool()? {
        args[1].clone()
    } else {
        args[2].clone()
    })
}
