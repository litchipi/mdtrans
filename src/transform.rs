use pest::{iterators::Pairs, Parser};
use std::collections::HashMap;

use crate::{errors::Errcode, MarkdownParser, Rule};

pub trait MarkdownTransformer {
    fn peek_text(&mut self, _text: &str) {}
    fn transform_text(&mut self, _text: String) -> String {
        "".to_string()
    }

    fn peek_header(&mut self, _level: usize, _header: &str) {}
    fn transform_header(&mut self, _level: usize, _text: String) -> String {
        "".to_string()
    }

    fn peek_bold(&mut self, _text: &str) {}
    fn transform_bold(&mut self, _text: String) -> String {
        "".to_string()
    }

    fn peek_italic(&mut self, _text: &str) {}
    fn transform_italic(&mut self, _text: String) -> String {
        "".to_string()
    }

    fn peek_link(&mut self, _text: &str, _url: &str) {}
    fn transform_link(&mut self, _text: String, _url: String) -> String {
        "".to_string()
    }

    fn peek_image(&mut self, _alt: &str, _url: &str, _add_tags: &HashMap<String, String>) {}
    fn transform_image(
        &mut self,
        _alt: String,
        _url: String,
        _add_tags: HashMap<String, String>,
    ) -> String {
        "".to_string()
    }

    fn peek_comment(&mut self, _text: String) {}
    fn transform_comment(&mut self, _text: String) -> String {
        "".to_string()
    }

    fn transform_strikethrough(&mut self) -> String {
        "".to_string()
    }
}

pub fn transform_markdown<F, O, T>(
    _input: F,
    _output: O,
    _transformer: &mut T,
) -> Result<String, Errcode>
where
    T: MarkdownTransformer,
    F: std::io::Read,
    O: std::io::Write,
{
    Ok("".to_string())
}

pub fn transform_markdown_string<T>(input: String, transformer: &mut T) -> Result<String, Errcode>
where
    T: MarkdownTransformer,
{
    let Some(parsed) = MarkdownParser::parse(Rule::file, &input)?.next() else {
        return Err(Errcode::ParsingError(
            "Parsed input returned an empty tree".to_string(),
        ));
    };

    let mut buffer = "".to_string();
    for node in parsed.into_inner() {
        let rule = node.as_rule();
        let inner = node.into_inner();
        if let Some(ref t) = forward_transformer(rule, inner, transformer) {
            buffer += t;
        }
    }

    Ok(buffer)
}

fn forward_transformer<T>(rule: Rule, mut inner: Pairs<Rule>, transformer: &mut T) -> Option<String>
where
    T: MarkdownTransformer,
{
    match rule {
        Rule::h1 => Some(transformer.transform_header(1, inner.next().unwrap().to_string())),
        Rule::h2 => Some(transformer.transform_header(2, inner.next().unwrap().to_string())),
        Rule::h3 => Some(transformer.transform_header(3, inner.next().unwrap().to_string())),
        Rule::h4 => Some(transformer.transform_header(4, inner.next().unwrap().to_string())),
        Rule::h5 => Some(transformer.transform_header(5, inner.next().unwrap().to_string())),
        Rule::h6 => Some(transformer.transform_header(6, inner.next().unwrap().to_string())),
        _ => None,
    }
}
