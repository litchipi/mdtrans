use mdtrans::{transform_markdown_string, MarkdownTransformer};

extern crate mdtrans;

const MD_POST_1: &str = include_str!("./data/post1.md");

#[derive(Default)]
pub struct Transformer {}

impl MarkdownTransformer for Transformer {
    // TODO    Implement this
}

fn main() {
    let mut transformer = Transformer::default();
    let res = transform_markdown_string(MD_POST_1.to_string(), &mut transformer).unwrap();
    println!("{res:?}");
}
