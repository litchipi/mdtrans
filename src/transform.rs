use std::collections::HashMap;

pub trait MarkdownTransformer {
    fn peek_text(&self, _text: &String) {}
    fn transform_text(&self, _text: String) -> String {
        "".to_string()
    }

    fn peek_header(&self, _level: usize, _header: &String) {}
    fn transform_header(&mut self, _level: usize, _text: String) -> String {
        "".to_string()
    }

    fn peek_bold(&self, _text: &String) {}
    fn transform_bold(&self, _text: String) -> String {
        "".to_string()
    }

    fn peek_italic(&self, _text: &String) {}
    fn transform_italic(&self, _text: String) -> String {
        "".to_string()
    }

    fn peek_link(&self, _text: &String, _url: &String) {}
    fn transform_link(&mut self, _text: String, _url: String) -> String {
        "".to_string()
    }

    fn peek_image(&self, _alt: &String, _url: &String, _add_tags: &HashMap<String, String>) {}
    fn transform_image(
        &mut self,
        _alt: String,
        _url: String,
        _add_tags: HashMap<String, String>,
    ) -> String {
        "".to_string()
    }

    fn peek_comment(&self, _text: String) {}
    fn transform_comment(&mut self, _text: String) -> String {
        "".to_string()
    }

    fn transform_strikethrough(&self) -> String {
        "".to_string()
    }
}
