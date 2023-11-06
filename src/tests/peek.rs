use std::collections::HashMap;

use crate::{transform_markdown_string, MarkdownTransformer};

#[test]
fn test_peek_reflink() {
    pub struct DummyTransform {
        refs: HashMap<String, String>,
    }
    impl MarkdownTransformer for DummyTransform {
        fn transform_reflink(&mut self, text: String, slug: String) -> String {
            let url = self.refs.get(&slug);
            assert!(url.is_some());
            format!("<a href=\"{}\">{text}</a>", url.unwrap())
        }
        fn transform_refurl(&mut self, _slug: String, _url: String) -> String {
            "".to_string()
        }
        fn peek_refurl(&mut self, slug: String, url: String) {
            self.refs.insert(slug, url);
        }
    }
    let mut t = DummyTransform {
        refs: HashMap::new(),
    };

    let res = transform_markdown_string("[a][b]\n[b]: c".to_string(), &mut t);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), "<a href=\"c\">a</a>".to_string());
}

// TODO    Test comments
