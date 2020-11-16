use syconf_lib::{Func, Value};

use crate::de::FUNC;
use crate::ser::to_value;
use crate::{from_value, Error};

use serde::{Deserialize, Deserializer};

#[derive(Deserialize, Debug)]
pub struct Function(#[serde(deserialize_with = "deserialize_func")] pub(crate) Func);

fn deserialize_func<'de, D>(_deser: D) -> Result<Func, D::Error>
where
    D: Deserializer<'de>,
{
    FUNC.with(|x| {
        let mut opt = x.borrow_mut();
        Ok((*opt).take().unwrap())
    })
}

impl Function {
    pub fn call1<I, O>(&self, input: &I) -> Result<O, Error>
    where
        I: serde::ser::Serialize,
        O: serde::de::DeserializeOwned,
    {
        let value: Value = to_value(input)?;
        let result = self.0.call(&[value])?;
        from_value(result).map_err(Into::into)
    }
}
