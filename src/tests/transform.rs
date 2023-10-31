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
        fn transform_header(&mut self, level: usize, _text: String) -> String {
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
