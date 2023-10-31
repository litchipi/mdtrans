use crate::{MarkdownTransformer, transform_markdown_string};

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
    assert_eq!(i.transform_comment("toto".to_string()), "tototutu".to_string());
}

#[test]
fn test_transform_string() {
    pub struct DummyTransform;
    impl MarkdownTransformer for DummyTransform {
        
    }
    let mut t = DummyTransform;
    let res = transform_markdown_string("".to_string(), &mut t);
    assert!(res.is_ok());
}
