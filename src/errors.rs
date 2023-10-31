#[derive(Debug)]
pub enum Errcode {
    ParsingError(String),
}

impl<T: std::fmt::Debug> From<pest::error::Error<T>> for Errcode {
    fn from(value: pest::error::Error<T>) -> Self {
        Errcode::ParsingError(format!("{:?}", value))
    }
}
