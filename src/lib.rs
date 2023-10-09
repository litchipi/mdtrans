use std::collections::HashMap;

pub trait MarkdownTransformer {
    // By default, do nothing
    fn transform_header(&mut self, _level: usize, _text: String) -> String {
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

    // By default, do nothing
    fn peek_header(&mut self, _level: usize, _header: &String) {}
}
