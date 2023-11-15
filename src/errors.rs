use thiserror::Error;

#[derive(Error, Debug)]
pub enum Errcode {
    ParsingError(String),
    IoError(std::io::Error),
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

impl From<std::io::Error> for Errcode {
    fn from(value: std::io::Error) -> Self {
        Errcode::IoError(value)
    }
}
