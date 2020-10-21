use crate::compiler::value::TypeMismatch;
use crate::compiler::Location;

pub type Error = ErrorWithLocation;
// pub type Error = anyhow::Error;

#[derive(thiserror::Error, Debug)]
pub struct ErrorWithLocation {
    pub location: Option<Location>,
    pub message: String,
}

impl std::fmt::Display for ErrorWithLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self
            .location
            .as_ref()
            .map(|l| format!("{}", l))
            .unwrap_or_else(|| "somewhere".to_string());

        write!(f, "{}: {}", &s, self.message)
    }
}

impl From<anyhow::Error> for ErrorWithLocation {
    fn from(e: anyhow::Error) -> Self {
        ErrorWithLocation {
            location: None,
            message: e.to_string(),
        }
    }
}

impl From<String> for ErrorWithLocation {
    fn from(e: String) -> Self {
        ErrorWithLocation {
            location: None,
            message: e,
        }
    }
}

impl From<&str> for ErrorWithLocation {
    fn from(e: &str) -> Self {
        ErrorWithLocation {
            location: None,
            message: e.to_string(),
        }
    }
}

impl From<TypeMismatch> for ErrorWithLocation {
    fn from(e: TypeMismatch) -> Self {
        ErrorWithLocation {
            location: None,
            message: e.to_string(),
        }
    }
}

#[macro_export]
macro_rules! check {
    ($cond:expr, $msg:literal $(,)?) => {
        if !$cond {
            return Err($crate::compiler::error::ErrorWithLocation{
                location: None,
                message: $msg.to_string(),
            });
        }
    };
    ($cond:expr, $fmt:expr, $($arg:tt)*) => {
        if !$cond {
            return Err($crate::compiler::error::ErrorWithLocation{
                location: None,
                message: format!($fmt, $($arg)*),
            });
        }
    };
}

#[test]
fn error_location() {
    let result = crate::parse_string(" abc");
    println!("Result: {:?}", &result);
    let err = result.err().unwrap();
    println!("Error: {}", &err);
    let loc = err.location;
    println!("Location: {:?}", loc);
    assert_eq!(loc.unwrap().position, 1);
}
