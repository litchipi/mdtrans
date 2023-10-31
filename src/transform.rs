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

    Ok(transform_pair(parsed, transformer))
}

fn next_inner_string(inner: &mut Pairs<Rule>) -> Option<String> {
    inner.next().map(|p| p.as_str().to_string())
}

fn transform_pair<T>(pair: Pair<Rule>, transformer: &mut T) -> String
where
    T: MarkdownTransformer,
{
    let rule = pair.as_rule();
    if let Rule::text = rule {
        return pair.as_str().to_string();
    }
    let mut inner = pair.into_inner();
    println!("Transform {rule:?}");
    match rule {
        Rule::h1 => transformer.transform_header(1, next_inner_string(&mut inner).unwrap()),
        Rule::h2 => transformer.transform_header(2, next_inner_string(&mut inner).unwrap()),
        Rule::h3 => transformer.transform_header(3, next_inner_string(&mut inner).unwrap()),
        Rule::h4 => transformer.transform_header(4, next_inner_string(&mut inner).unwrap()),
        Rule::h5 => transformer.transform_header(5, next_inner_string(&mut inner).unwrap()),
        Rule::h6 => transformer.transform_header(6, next_inner_string(&mut inner).unwrap()),
        Rule::italic => transformer.transform_italic(next_inner_string(&mut inner).unwrap()),
        Rule::bold => transformer.transform_bold(next_inner_string(&mut inner).unwrap()),
        Rule::link => {
            let text = inner.next().unwrap();
            let text = transform_pair(text, transformer);
            let url = inner.next().unwrap().as_str().to_string();
            transformer.transform_link(text, url)
        }
        Rule::file | Rule::rich_txt => {
            let mut buffer = "".to_string();
            for child in inner {
                buffer += transform_pair(child, transformer).as_str();
            }
            println!("rich text {buffer}");
            buffer
        }
        Rule::EOI => "".to_string(),
        r => {
            println!("{r:?} not implemented");
            "".to_string()
        }
    }
}
