use syconf_lib::{parse_file, parse_string, ErrorWithLocation, TypeMismatch, Value};

use crate::de::Deserializer;
pub use de::Function;

mod de;
mod ser;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{}", .0)]
    ConfigError(#[from] ErrorWithLocation),
    #[error("{}", .0)]
    TypeMismatch(#[from] TypeMismatch),
    #[error("Mapping error: {}", .0)]
    Custom(String),
    #[error("Unsupported type")]
    UnsupportedType,
}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Error::Custom(msg.to_string())
    }
}
impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Error::Custom(msg.to_string())
    }
}

pub fn from_str<T>(s: &str) -> Result<T, Error>
where
    T: serde::de::DeserializeOwned,
{
    from_value(parse_string(s)?)
}

pub fn from_file<T>(file_name: &str) -> Result<T, Error>
where
    T: serde::de::DeserializeOwned,
{
    from_value(parse_file(file_name)?)
}

fn from_value<T>(value: Value) -> Result<T, Error>
where
    T: serde::de::DeserializeOwned,
{
    T::deserialize(Deserializer::new(value))
}
