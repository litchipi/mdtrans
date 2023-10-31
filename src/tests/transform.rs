use crate::{transform_markdown_string, MarkdownTransformer};

#[test]
fn test_trait_impl() {
    pub struct TestImpl;
    impl MarkdownTransformer for TestImpl {
        fn transform_comment(&mut self, mut orig: String) -> String {
            orig += "tutu";
            orig
        }
    }

    let mut i = TestImpl;
    assert_eq!(
        i.transform_comment("toto".to_string()),
        "tototutu".to_string()
    );
}

#[test]
fn test_transform_string() {
    pub struct DummyTransform;
    impl MarkdownTransformer for DummyTransform {}
    let mut t = DummyTransform;
    let res = transform_markdown_string("".to_string(), &mut t);
    assert!(res.is_ok());
}

#[test]
fn test_transform_header() {
    pub struct DummyTransform;
    impl MarkdownTransformer for DummyTransform {
        fn transform_header(&mut self, level: usize, _: String) -> String {
            format!("h{level}")
        }
    }
    let mut t = DummyTransform;

    for level in 1..7 {
        let res = transform_markdown_string(format!("{} header", "#".repeat(level)), &mut t);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), format!("h{level}"));
    }
}

#[test]
fn test_transform_italic() {
    pub struct DummyTransform;
    impl MarkdownTransformer for DummyTransform {
        fn transform_italic(&mut self, _: String) -> String {
            "italic".to_string()
        }
    }
    let mut t = DummyTransform;

    let res = transform_markdown_string("*toto*".to_string(), &mut t);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), "italic".to_string());
}

#[test]
fn test_transform_bold() {
    pub struct DummyTransform;
    impl MarkdownTransformer for DummyTransform {
        fn transform_bold(&mut self, _: String) -> String {
            "bold".to_string()
        }
    }
    let mut t = DummyTransform;

    let res = transform_markdown_string("**toto**".to_string(), &mut t);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), "bold".to_string());
}

#[test]
fn test_transform_link() {
    pub struct DummyTransform;
    impl MarkdownTransformer for DummyTransform {
        fn transform_link(&mut self, text: String, url: String) -> String {
            format!("{text}: {url}")
        }
    }
    let mut t = DummyTransform;

    let res = transform_markdown_string("[a](b)".to_string(), &mut t);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), "a: b".to_string());
}
