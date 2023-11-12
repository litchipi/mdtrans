mod errors;
mod transform;

#[cfg(test)]
mod tests;

use pest_derive::Parser;
pub use transform::*;
pub use errors::Errcode;

#[derive(Parser)]
#[grammar = "markdown.pest"]
pub struct MarkdownParser;
