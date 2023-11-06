use pest::{
    iterators::{Pair, Pairs},
    Parser,
};
use std::{collections::HashMap, unimplemented};

use crate::{errors::Errcode, MarkdownParser, Rule};

pub trait MarkdownTransformer {
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

    fn peek_image(&mut self, _alt: String, _url: String, _add_tags: HashMap<String, String>) {}
    fn transform_image(
        &mut self,
        _alt: String,
        _url: String,
        _add_tags: HashMap<String, String>,
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

    let mut state = ParseState::new(transformer);
    state.peek_pair(parsed.clone());
    let mut state = ParseState::new(transformer);
    Ok(state.transform_pair(parsed))
}

fn next_inner_string(inner: &mut Pairs<Rule>) -> Option<String> {
    inner.next().map(|p| p.as_str().to_string())
}

pub struct ParseState<'a, T> {
    in_quote: bool,
    buffers: [String; 1],
    transformer: &'a mut T,
}

impl<'a, T> ParseState<'a, T>
where
    T: MarkdownTransformer,
{
    fn new(transformer: &mut T) -> ParseState<T> {
        ParseState {
            in_quote: false,
            buffers: [String::new(); 1],
            transformer,
        }
    }

    fn peek_pair(&mut self, pair: Pair<Rule>) {
        let rule = pair.as_rule();
        if let Rule::text = rule {
            let text = pair.as_str().to_string();
            if self.in_quote {
                let buffer = self.buffers.get_mut(0).unwrap();
                if text.is_empty() {
                    self.in_quote = false;
                    let quote_text = buffer.clone();
                    self.transformer.peek_quote(quote_text);
                    *buffer = String::new();
                } else {
                    *buffer += text.as_str();
                    *buffer += "\n";
                }
            }
            self.transformer.peek_text(pair.as_str().to_string());
            return;
        }
        let mut inner = pair.into_inner();
        println!("Peek {rule:?}");
        match rule {
            Rule::h1 => self
                .transformer
                .peek_header(1, next_inner_string(&mut inner).unwrap()),
            Rule::h2 => self
                .transformer
                .peek_header(2, next_inner_string(&mut inner).unwrap()),
            Rule::h3 => self
                .transformer
                .peek_header(3, next_inner_string(&mut inner).unwrap()),
            Rule::h4 => self
                .transformer
                .peek_header(4, next_inner_string(&mut inner).unwrap()),
            Rule::h5 => self
                .transformer
                .peek_header(5, next_inner_string(&mut inner).unwrap()),
            Rule::h6 => self
                .transformer
                .peek_header(6, next_inner_string(&mut inner).unwrap()),
            Rule::italic => self
                .transformer
                .peek_italic(next_inner_string(&mut inner).unwrap()),
            Rule::bold => self
                .transformer
                .peek_bold(next_inner_string(&mut inner).unwrap()),
            Rule::link => {
                let text = next_inner_string(&mut inner).unwrap();
                let url = next_inner_string(&mut inner).unwrap();
                self.transformer.peek_link(text, url)
            }
            Rule::reflink => {
                let text = next_inner_string(&mut inner).unwrap();
                let slug = next_inner_string(&mut inner).unwrap();
                self.transformer.peek_reflink(text, slug)
            }
            Rule::refurl => {
                let slug = next_inner_string(&mut inner).unwrap();
                let url = next_inner_string(&mut inner).unwrap();
                self.transformer.peek_refurl(slug, url)
            }
            Rule::file | Rule::rich_txt => {
                for child in inner {
                    self.peek_pair(child);
                }
            }
            Rule::quote => {
                let quote_text = next_inner_string(&mut inner).unwrap();
                self.in_quote = true;
                *self.buffers.get_mut(0).unwrap() += quote_text.as_str();
            }
            Rule::codeblock => {
                let code_text = next_inner_string(&mut inner).unwrap();
                self.transformer.peek_codeblock(code_text)
            }
            Rule::code => todo!(),
            Rule::horiz_sep => todo!(),
            r => {
                println!("{r:?} not implemented");
            }
        }
    }

    fn transform_raw_text(&mut self, text: String) -> String {
        if self.in_quote {
            if text.is_empty() {
                self.in_quote = false;
                let quote_text = self.buffers.get(0).unwrap().clone();
                *self.buffers.get_mut(0).unwrap() = String::new();
                return self.transformer.transform_quote(quote_text);
            } else {
                *self.buffers.get_mut(0).unwrap() += text.as_str();
                return "".to_string();
            }
        }
        self.transformer.transform_text(text)
    }

    fn is_block_type(&self, rule: &Rule) -> bool {
        matches!(
            rule,
            Rule::h1
                | Rule::h2
                | Rule::h3
                | Rule::h4
                | Rule::h5
                | Rule::h6
                | Rule::codeblock
                | Rule::comment
        )
    }

    fn transform_pair(&mut self, pair: Pair<Rule>) -> String {
        let rule = pair.as_rule();
        if let Rule::text = rule {
            let text = pair.as_str().to_string();
            return self.transform_raw_text(text);
        }
        let mut inner = pair.into_inner();
        println!("Transform {rule:?}");
        let add_newline = self.is_block_type(&rule);
        let text = match rule {
            Rule::h1 => self
                .transformer
                .transform_header(1, next_inner_string(&mut inner).unwrap()),
            Rule::h2 => self
                .transformer
                .transform_header(2, next_inner_string(&mut inner).unwrap()),
            Rule::h3 => self
                .transformer
                .transform_header(3, next_inner_string(&mut inner).unwrap()),
            Rule::h4 => self
                .transformer
                .transform_header(4, next_inner_string(&mut inner).unwrap()),
            Rule::h5 => self
                .transformer
                .transform_header(5, next_inner_string(&mut inner).unwrap()),
            Rule::h6 => self
                .transformer
                .transform_header(6, next_inner_string(&mut inner).unwrap()),
            Rule::italic => self
                .transformer
                .transform_italic(next_inner_string(&mut inner).unwrap()),
            Rule::bold => self
                .transformer
                .transform_bold(next_inner_string(&mut inner).unwrap()),
            Rule::link => {
                let text = inner.next().unwrap();
                let text = self.transform_pair(text);
                let url = next_inner_string(&mut inner).unwrap();
                self.transformer.transform_link(text, url)
            }
            Rule::reflink => {
                let text = inner.next().unwrap();
                let text = self.transform_pair(text);
                let slug = next_inner_string(&mut inner).unwrap();
                self.transformer.transform_reflink(text, slug)
            }
            Rule::refurl => {
                let slug = next_inner_string(&mut inner).unwrap();
                let url = next_inner_string(&mut inner).unwrap();
                self.transformer.transform_refurl(slug, url)
            }
            Rule::quote => {
                let text = inner.next().unwrap();
                let text = self.transform_pair(text);
                self.in_quote = true;
                let buffer = self.buffers.get_mut(0).unwrap();
                *buffer += text.as_str();
                *buffer += "\n";
                "".to_string()
            }
            Rule::codeblock => {
                let code_text = next_inner_string(&mut inner).unwrap();
                self.transformer.transform_codeblock(code_text)
            }
            Rule::code => todo!(),
            Rule::horiz_sep => todo!(),
            Rule::file | Rule::rich_txt => {
                let mut buffer = "".to_string();
                if inner.len() == 0 {
                    return self.transform_raw_text(inner.as_str().to_string());
                }
                for child in inner {
                    buffer += self.transform_pair(child).as_str();
                }
                println!("rich text {buffer}");
                buffer
            }
            r => {
                println!("{r:?} not implemented");
                "".to_string()
            }
        };
        if add_newline {
            "\n".to_string() + text.as_str() + "\n"
        } else {
            text
        }
    }
}
