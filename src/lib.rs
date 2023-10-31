mod errors;
mod transform;

#[cfg(test)]
mod tests;

use pest_derive::Parser;
pub use transform::*;

#[derive(Parser)]
#[grammar = "markdown.pest"]
pub struct MarkdownParser;
