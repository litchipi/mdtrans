mod errors;
mod transform;

#[cfg(test)]
mod tests;

pub use errors::Errcode;
use pest_derive::Parser;
pub use transform::*;

#[derive(Parser)]
#[grammar = "markdown.pest"]
pub struct MarkdownParser;
