mod transform;
mod errors;

#[cfg(test)]
mod tests;

use pest_derive::Parser;
pub use transform::*;

#[derive(Parser)]
#[grammar = "markdown.pest"]
pub struct MarkdownParser;

