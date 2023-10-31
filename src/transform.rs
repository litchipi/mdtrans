use std::collections::HashMap;
use crate::errors::Errcode;


pub trait MarkdownTransformer {
    fn peek_text(&mut self, _text: &String) {}
    fn transform_text(&mut self, _text: String) -> String {
        "".to_string()
    }

    fn peek_header(&mut self, _level: usize, _header: &String) {}
    fn transform_header(&mut self, _level: usize, _text: String) -> String {
        "".to_string()
    }

    fn peek_bold(&mut self, _text: &String) {}
    fn transform_bold(&mut self, _text: String) -> String {
        "".to_string()
    }

    fn peek_italic(&mut self, _text: &String) {}
    fn transform_italic(&mut self, _text: String) -> String {
        "".to_string()
    }

    fn peek_link(&mut self, _text: &String, _url: &String) {}
    fn transform_link(&mut self, _text: String, _url: String) -> String {
        "".to_string()
    }

    fn peek_image(&mut self, _alt: &String, _url: &String, _add_tags: &HashMap<String, String>) {}
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

pub fn transform_markdown<F, O, T>(_input: F, _output: O, _transformer: &mut T) -> Result<String, Errcode> where 
    T: MarkdownTransformer,
    F: std::io::Read,
    O: std::io::Write,
{
    Ok("".to_string())
    
}

pub fn transform_markdown_string<T>(_input: String, _transformer: &mut T) -> Result<String, Errcode> where 
    T: MarkdownTransformer,
{
    Ok("".to_string())
    
}
