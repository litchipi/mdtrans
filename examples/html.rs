use mdtrans::{transform_markdown_string, MarkdownTransformer};

extern crate mdtrans;

const MD_POST_1: &str = include_str!("./data/post1.md");

#[derive(Default)]
pub struct Transformer {}

impl MarkdownTransformer for Transformer {
    fn peek_text(&mut self, _text: String) {}

    fn transform_text(&mut self, text: String) -> String {
        text
    }

    fn peek_header(&mut self, _level: usize, _text: String) {}

    fn transform_header(&mut self, _level: usize, _text: String) -> String {
        unimplemented!()
    }

    fn peek_bold(&mut self, _text: String) {}

    fn transform_bold(&mut self, _text: String) -> String {
        unimplemented!()
    }

    fn peek_italic(&mut self, _text: String) {}

    fn transform_italic(&mut self, _text: String) -> String {
        unimplemented!()
    }

    fn peek_reflink(&mut self, _text: String, _slug: String) {}

    fn transform_reflink(&mut self, _text: String, _slug: String) -> String {
        unimplemented!()
    }

    fn peek_refurl(&mut self, _slug: String, _url: String) {}

    fn transform_refurl(&mut self, _slug: String, _url: String) -> String {
        unimplemented!()
    }

    fn peek_link(&mut self, _text: String, _url: String) {}

    fn transform_link(&mut self, _text: String, _url: String) -> String {
        unimplemented!()
    }

    fn peek_image(
        &mut self,
        _alt: String,
        _url: String,
        _add_tags: std::collections::HashMap<String, String>,
    ) {
    }

    fn transform_image(
        &mut self,
        _alt: String,
        _url: String,
        _add_tags: std::collections::HashMap<String, String>,
    ) -> String {
        unimplemented!()
    }

    fn peek_comment(&mut self, _text: String) {}

    fn transform_comment(&mut self, _text: String) -> String {
        unimplemented!()
    }

    fn peek_strikethrough(&mut self) {}

    fn transform_strikethrough(&mut self) -> String {
        unimplemented!()
    }

    fn peek_quote(&mut self, _text: String) {}

    fn transform_quote(&mut self, _text: String) -> String {
        unimplemented!()
    }

    fn peek_codeblock(&mut self, _text: String) {}

    fn transform_codeblock(&mut self, _text: String) -> String {
        unimplemented!()
    }

    fn peek_inline_code(&mut self, _text: String) {}

    fn transform_inline_code(&mut self, _text: String) -> String {
        unimplemented!()
    }

    fn peek_horizontal_separator(&mut self) {}

    fn transform_horizontal_separator(&mut self) -> String {
        unimplemented!()
    }
}

fn main() {
    let mut transformer = Transformer::default();
    let res = transform_markdown_string(MD_POST_1.to_string(), &mut transformer).unwrap();
    println!("{res:?}");
}
