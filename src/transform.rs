use pest::{
    iterators::{Pair, Pairs},
    Parser,
};
use std::collections::HashMap;

use crate::{errors::Errcode, MarkdownParser, Rule};

pub trait MarkdownTransformer {
    fn transform_text(&mut self, _text: String) -> String {
        "".to_string()
    }

    fn transform_header(&mut self, _level: usize, _text: String) -> String {
        "".to_string()
    }

    fn transform_bold(&mut self, _text: String) -> String {
        "".to_string()
    }

    fn transform_italic(&mut self, _text: String) -> String {
        "".to_string()
    }

    fn transform_link(&mut self, _text: String, _url: String) -> String {
        "".to_string()
    }

    fn transform_image(
        &mut self,
        _alt: String,
        _url: String,
        _add_tags: HashMap<String, String>,
    ) -> String {
        "".to_string()
    }

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

    Ok(transform_pair(
        parsed.as_rule(),
        parsed.into_inner(),
        transformer,
    ))
}

fn transform_pair<T>(rule: Rule, mut inner: Pairs<Rule>, transformer: &mut T) -> String
where
    T: MarkdownTransformer,
{
    match rule {
        Rule::h1 => transformer.transform_header(1, inner.next().unwrap().to_string()),
        Rule::h2 => transformer.transform_header(2, inner.next().unwrap().to_string()),
        Rule::h3 => transformer.transform_header(3, inner.next().unwrap().to_string()),
        Rule::h4 => transformer.transform_header(4, inner.next().unwrap().to_string()),
        Rule::h5 => transformer.transform_header(5, inner.next().unwrap().to_string()),
        Rule::h6 => transformer.transform_header(6, inner.next().unwrap().to_string()),
        Rule::italic => transformer.transform_italic(inner.next().unwrap().to_string()),
        Rule::bold => transformer.transform_bold(inner.next().unwrap().to_string()),
        Rule::link => {
            let text = inner.next().unwrap().as_str().to_string();
            let url = inner.next().unwrap().as_str().to_string();
            transformer.transform_link(text, url)
        }
        Rule::file | Rule::rich_txt => {
            let mut buffer = "".to_string();
            for child in inner {
                buffer += transform_pair(child.as_rule(), child.into_inner(), transformer).as_str();
            }
            buffer
        }
        r => {
            println!("{r:?} not implemented");
            "".to_string()
        }
    }
}
