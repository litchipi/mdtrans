use thiserror::Error;

// TODO    Remove all confort unwrap in the code, replace with recoverable errors
#[derive(Error, Debug)]
pub enum Errcode {
    ParsingError(String),
}

impl std::fmt::Display for Errcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<T: std::fmt::Debug> From<pest::error::Error<T>> for Errcode {
    fn from(value: pest::error::Error<T>) -> Self {
        Errcode::ParsingError(format!("{:?}", value))
    }
}
