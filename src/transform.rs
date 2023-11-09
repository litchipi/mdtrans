use pest::{
    iterators::{Pair, Pairs},
    Parser,
};
use std::{collections::HashMap, unimplemented};

use crate::{errors::Errcode, MarkdownParser, Rule};

pub trait MarkdownTransformer {
    fn reset(&mut self) {}

    fn peek_text(&mut self, _text: String) {}
    fn transform_text(&mut self, text: String) -> String {
        text
    }

    fn peek_header(&mut self, _level: usize, _text: String) {}
    fn transform_header(&mut self, _level: usize, _text: String) -> String {
        unimplemented!("header")
    }

    fn peek_bold(&mut self, _text: String) {}
    fn transform_bold(&mut self, _text: String) -> String {
        unimplemented!("bold")
    }

    fn peek_italic(&mut self, _text: String) {}
    fn transform_italic(&mut self, _text: String) -> String {
        unimplemented!("italic")
    }

    fn peek_reflink(&mut self, _text: String, _slug: String) {}
    fn transform_reflink(&mut self, _text: String, _slug: String) -> String {
        unimplemented!("reflink")
    }

    fn peek_refurl(&mut self, _slug: String, _url: String) {}
    fn transform_refurl(&mut self, _slug: String, _url: String) -> String {
        unimplemented!("refurl")
    }

    fn peek_link(&mut self, _text: String, _url: String) {}
    fn transform_link(&mut self, _text: String, _url: String) -> String {
        unimplemented!("link")
    }

    fn peek_image(&mut self, _alt: String, _url: String, _add_tags: HashMap<String, String>) {}
    fn transform_image(
        &mut self,
        _alt: String,
        _url: String,
        _add_tags: HashMap<String, String>,
    ) -> String {
        unimplemented!("image")
    }

    fn peek_comment(&mut self, _text: String) {}
    fn transform_comment(&mut self, _text: String) -> String {
        unimplemented!("comment")
    }

    fn peek_strikethrough(&mut self) {}
    fn transform_strikethrough(&mut self) -> String {
        unimplemented!("strikethrough")
    }

    fn peek_quote(&mut self, _text: String) {}
    fn transform_quote(&mut self, _text: String) -> String {
        unimplemented!("quote")
    }

    fn peek_codeblock(&mut self, _language: Option<String>, _text: String) {}
    fn transform_codeblock(&mut self, _language: Option<String>, _text: String) -> String {
        unimplemented!("codeblock")
    }

    fn peek_inline_code(&mut self, _text: String) {}
    fn transform_inline_code(&mut self, _text: String) -> String {
        unimplemented!("inline code")
    }

    fn peek_horizontal_separator(&mut self) {}
    fn transform_horizontal_separator(&mut self) -> String {
        unimplemented!("horizontal separator")
    }

    fn peek_list(&mut self, _elements: Vec<String>) {}
    fn transform_list(&mut self, _elements: Vec<String>) -> String {
        unimplemented!("list")
    }

    fn peek_list_element(&mut self, _element: String) {}
    fn transform_list_element(&mut self, _element: String) -> String {
        unimplemented!("list element")
    }

    fn peek_vertical_space(&mut self) {}
    fn transform_vertical_space(&mut self) -> String {
        "\n".to_string()
    }

    fn peek_paragraph(&mut self, _text: String) {}
    fn transform_paragraph(&mut self, text: String) -> String {
        text
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

    let mut parser = TransformFramework::new(transformer);
    parser.act_on_pair(&mut ParseState::peek(), parsed.clone());
    Ok(parser.act_on_pair(&mut ParseState::default(), parsed))
}

fn next_inner_string(inner: &mut Pairs<Rule>) -> Option<String> {
    inner.next().map(|p| p.as_str().to_string())
}

#[derive(Default, Clone)]
pub struct ParseState {
    peek: bool,
    add_space: bool,
}

impl ParseState {
    pub fn peek() -> ParseState {
        let default = Self::default();
        ParseState { peek: true, ..default }
    }
}

struct TransformFramework<'a, T> {
    transformer: &'a mut T,
}

impl<'a, T> TransformFramework<'a, T>
where
    T: MarkdownTransformer,
{
    fn new(transformer: &mut T) -> TransformFramework<T> {
        TransformFramework {
            transformer,
        }
    }

    fn get_rich_text(&mut self, state: &mut ParseState, nb: usize, inner: &mut Pairs<Rule>) -> String {
        assert!(nb <= inner.len());
        let mut child_state = state.clone();
        let inners = (0..nb)
            .map(|_| {
                let pair = inner.next().unwrap();
                self.act_on_pair(&mut child_state, pair)
            })
            .collect::<Vec<String>>();
        inners.join("")
    }

    fn act_on_raw_text(&mut self, state: &mut ParseState, text: String) -> String {
        if state.peek {
            self.transformer.peek_text(text);
            "".to_string()
        } else {
            self.transformer.transform_text(text)
        }
    }

    fn needs_newline_sep(&self, rule: &Rule) -> bool {
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
                | Rule::horiz_sep
                | Rule::list
        )
    }

    fn get_whole_block(&self, inner: &mut Pairs<Rule>, join: &str) -> String {
        let mut buffer = "".to_string();
        for code_line in inner {
            buffer += code_line.as_str();
            buffer += join;
        }
        let end = buffer.len() - join.len();
        buffer[..end].to_string()
    }

    fn act_on_pair(&mut self, state: &mut ParseState, pair: Pair<Rule>) -> String {
        let mut text: String = "".to_string();
        let rule = pair.as_rule();
        if matches!(rule, Rule::text | Rule::link_text | Rule::code) {
            if state.add_space {
                text += " ";
                state.add_space = false;
            }
            let raw_text = pair.as_str().to_string();
            text += self.act_on_raw_text(state, raw_text).as_str();
            return text;
        }
        let mut inner = pair.into_inner();
        let add_newline = self.needs_newline_sep(&rule);
        match rule {
            Rule::h1 if state.peek => self
                .transformer
                .peek_header(1, next_inner_string(&mut inner).unwrap()),
            Rule::h1 => {
                text = self
                    .transformer
                    .transform_header(1, next_inner_string(&mut inner).unwrap())
            }

            Rule::h2 if state.peek => self
                .transformer
                .peek_header(2, next_inner_string(&mut inner).unwrap()),
            Rule::h2 => {
                text = self
                    .transformer
                    .transform_header(2, next_inner_string(&mut inner).unwrap())
            }

            Rule::h3 if state.peek => self
                .transformer
                .peek_header(3, next_inner_string(&mut inner).unwrap()),
            Rule::h3 => {
                text = self
                    .transformer
                    .transform_header(3, next_inner_string(&mut inner).unwrap())
            }

            Rule::h4 if state.peek => self
                .transformer
                .peek_header(4, next_inner_string(&mut inner).unwrap()),
            Rule::h4 => {
                text = self
                    .transformer
                    .transform_header(4, next_inner_string(&mut inner).unwrap())
            }

            Rule::h5 if state.peek => self
                .transformer
                .peek_header(5, next_inner_string(&mut inner).unwrap()),
            Rule::h5 => {
                text = self
                    .transformer
                    .transform_header(5, next_inner_string(&mut inner).unwrap())
            }

            Rule::h6 if state.peek => self
                .transformer
                .peek_header(6, next_inner_string(&mut inner).unwrap()),
            Rule::h6 => {
                text = self
                    .transformer
                    .transform_header(6, next_inner_string(&mut inner).unwrap())
            }

            Rule::italic if state.peek => self
                .transformer
                .peek_italic(next_inner_string(&mut inner).unwrap()),
            Rule::italic => {
                text = self
                    .transformer
                    .transform_italic(next_inner_string(&mut inner).unwrap())
            }

            Rule::bold if state.peek => self
                .transformer
                .peek_bold(next_inner_string(&mut inner).unwrap()),
            Rule::bold => {
                text = self
                    .transformer
                    .transform_bold(next_inner_string(&mut inner).unwrap())
            }

            Rule::link => {
                let link_text = self.get_rich_text(state, inner.len() - 1, &mut inner);
                let url = next_inner_string(&mut inner).unwrap();
                if state.peek {
                    self.transformer.peek_link(link_text, url);
                } else {
                    text = self.transformer.transform_link(link_text, url);
                }
            }
            Rule::reflink => {
                let link_text = self.get_rich_text(state, inner.len() - 1, &mut inner);
                let slug = next_inner_string(&mut inner).unwrap();
                if state.peek {
                    self.transformer.peek_reflink(link_text, slug);
                } else {
                    text = self.transformer.transform_reflink(link_text, slug);
                }
            }
            Rule::refurl => {
                let slug = next_inner_string(&mut inner).unwrap();
                let url = next_inner_string(&mut inner).unwrap();
                if state.peek {
                    self.transformer.peek_refurl(slug, url);
                } else {
                    text = self.transformer.transform_refurl(slug, url);
                }
            }
            Rule::quote => {
                let lines = inner
                    .map(|line| {
                        assert_eq!(line.as_rule(), Rule::quote_line);
                        self.act_on_pair(state, line)
                    })
                    .collect::<Vec<String>>();
                let quote_text = lines.join("\n");
                if state.peek {
                    self.transformer.peek_quote(quote_text);
                } else {
                    text = self.transformer.transform_quote(quote_text);
                }
            }
            Rule::quote_line => {
                text = self.get_rich_text(state, inner.len(), &mut inner);
            }
            Rule::codeblock => {
                let mut got_lang = false;
                if let Some(t) = inner.peek() {
                    if t.as_rule() == Rule::slug {
                        got_lang = true;
                    }
                }
                let lang = if got_lang {
                    Some(inner.next().unwrap().as_str().to_string())
                } else {
                    None
                };
                if state.peek {
                    self
                        .transformer
                        .peek_codeblock(lang, self.get_whole_block(&mut inner, "\n"));
                } else {
                    text = self
                        .transformer
                        .transform_codeblock(lang, self.get_whole_block(&mut inner, "\n"))
                }
            },
            Rule::inline_code => {
                let code_text = next_inner_string(&mut inner).unwrap();
                if state.peek {
                    self.transformer.peek_inline_code(code_text)
                } else {
                    text = self.transformer.transform_inline_code(code_text)
                }
            }
            Rule::horiz_sep if state.peek => self.transformer.peek_horizontal_separator(),
            Rule::horiz_sep => text = self.transformer.transform_horizontal_separator(),
            Rule::file | Rule::rich_txt | Rule::quote_txt => {
                if inner.len() == 0 {
                    return self.act_on_raw_text(state, inner.as_str().to_string());
                }
                for child in inner {
                    text += self.act_on_pair(state, child).as_str();
                }
            }
            Rule::EOI => {}
            Rule::image => {
                let img_alt = next_inner_string(&mut inner).unwrap();
                let url = next_inner_string(&mut inner).unwrap();
                let added_tags = HashMap::new(); // TODO    Added tags
                if state.peek {
                    self.transformer.peek_image(img_alt, url, added_tags);
                } else {
                    text = self.transformer.transform_image(img_alt, url, added_tags);
                }
            }
            Rule::list => {
                let elements: Vec<String> = inner.map(|el| self.act_on_pair(state, el)).collect();
                if state.peek {
                    self.transformer.peek_list(elements);
                } else {
                    text = self.transformer.transform_list(elements);
                }
            }
            Rule::list_element => {
                let element_text = self.get_rich_text(state, inner.len(), &mut inner);
                if state.peek {
                    self.transformer.peek_list_element(element_text);
                } else {
                    text = self.transformer.transform_list_element(element_text);
                }
            }
            Rule::paragraph_newline => {
                state.add_space = true
            },
            Rule::paragraph => {
                let paragraph_text = self.get_rich_text(state, inner.len(), &mut inner);
                if state.peek {
                    self.transformer.peek_paragraph(paragraph_text);
                } else {
                    text += self.transformer.transform_paragraph(paragraph_text).as_str();
                }
            }
            Rule::vertical_space => if state.peek {
                self.transformer.peek_vertical_space()
            } else {
                text = self.transformer.transform_vertical_space();
            }
            r => unimplemented!("{r:?}"),
        };
        if add_newline {
            "\n".to_string() + text.as_str() + "\n"
        } else {
            text
        }
    }
}
