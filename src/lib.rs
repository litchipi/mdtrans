use std::collections::HashMap;

pub trait MarkdownTransformer {
    // By default, do nothing
    fn transform_header(&mut self, _level: usize, _text: String) -> String {
        "".to_string()
    }

    fn transform_link(&mut self, text: String, url: String) -> String {
        "".to_string()
    }
    fn transform_image(
        &mut self,
        alt: String,
        url: String,
        add_tags: HashMap<String, String>,
    ) -> String {
        "".to_string()
    }
    fn transform_comment(&mut self, text: String) -> String {
        "".to_string()
    }

    // By default, do nothing
    fn peek_header(&mut self, level: usize, header: &String) {}
}
