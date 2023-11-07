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
    assert!(res.is_ok(), "Error on transformation: {res:?}");
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
        let res =
            transform_markdown_string(format!("start\n{} header\nend", "#".repeat(level)), &mut t);
        assert!(res.is_ok(), "Error on transformation: {res:?}");
        assert_eq!(res.unwrap(), format!("start\nh{level}\nend"));
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
    assert!(res.is_ok(), "Error on transformation: {res:?}");
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
    assert!(res.is_ok(), "Error on transformation: {res:?}");
    assert_eq!(res.unwrap(), "bold".to_string());
}

#[test]
fn test_transform_link() {
    pub struct DummyTransform;
    impl MarkdownTransformer for DummyTransform {
        fn transform_link(&mut self, text: String, url: String) -> String {
            format!("{text}: {url}")
        }
        fn transform_bold(&mut self, text: String) -> String {
            text
        }
    }
    let mut t = DummyTransform;

    let res = transform_markdown_string("[a](b)".to_string(), &mut t);
    assert!(res.is_ok(), "Error on transformation: {res:?}");
    assert_eq!(res.unwrap(), "a: b".to_string());

    let res = transform_markdown_string("[a **bold** c](b)".to_string(), &mut t);
    assert!(res.is_ok(), "Error on transformation: {res:?}");
    assert_eq!(res.unwrap(), "a bold c: b".to_string());
}

#[test]
fn test_transform_quote() {
    pub struct DummyTransform;
    impl MarkdownTransformer for DummyTransform {
        fn transform_quote(&mut self, text: String) -> String {
            println!("QUOTE TEXT: {text}");
            format!("QUOTE\n{text}\nQUOTE")
        }
    }
    let mut t = DummyTransform;

    let input = "> Je suis une truite\nJe suis un saumon\n\n";
    let output = "QUOTE\nJe suis une truite\nJe suis un saumon\nQUOTE";
    let res = transform_markdown_string(input.to_string(), &mut t);
    assert!(res.is_ok(), "Error on transformation: {res:?}");
    assert_eq!(res.unwrap(), output.to_string());
}

#[test]
fn test_transform_codeblock() {
    pub struct DummyTransform;
    impl MarkdownTransformer for DummyTransform {
        fn transform_codeblock(&mut self, text: String) -> String {
            format!("CODEBLOCK\n{text}\nCODEBLOCK")
        }
    }
    let mut t = DummyTransform;

    let input = "start\n```\nsome\ncode\n```\nend";
    let output = "start\nCODEBLOCK\nsome\ncode\nCODEBLOCK\nend";
    let res = transform_markdown_string(input.to_string(), &mut t);
    assert!(res.is_ok(), "Error on transformation: {res:?}");
    assert_eq!(res.unwrap(), output.to_string());
}

#[test]
fn test_transform_inline_code() {
    pub struct DummyTransform;
    impl MarkdownTransformer for DummyTransform {
        fn transform_inline_code(&mut self, text: String) -> String {
            format!("CODE {text} CODE")
        }
    }
    let mut t = DummyTransform;

    let input = "start `some code` end";
    let output = "start CODE some code CODE end";
    let res = transform_markdown_string(input.to_string(), &mut t);
    assert!(res.is_ok(), "Error on transformation: {res:?}");
    assert_eq!(res.unwrap(), output.to_string());
}

#[test]
fn test_transform_horiz_sep() {
    pub struct DummyTransform;
    impl MarkdownTransformer for DummyTransform {
        fn transform_horizontal_separator(&mut self) -> String {
            "=== HORIZ SEPARATOR ===".to_string()
        }
    }
    let mut t = DummyTransform;

    let input = "start\n---\nend";
    let output = "start\n=== HORIZ SEPARATOR ===\nend";
    let res = transform_markdown_string(input.to_string(), &mut t);
    assert!(res.is_ok(), "Error on transformation: {res:?}");
    assert_eq!(res.unwrap(), output.to_string());
}

#[test]
fn test_transform_list() {
    pub struct DummyTransform;
    impl MarkdownTransformer for DummyTransform {
        fn transform_list(&mut self, elements: Vec<String>) -> String {
            elements.join(", ")
        }

        fn transform_bold(&mut self, text: String) -> String {
            format!("BOLD {text} BOLD")
        }

        fn transform_italic(&mut self, text: String) -> String {
            format!("ITALIC {text} ITALIC")
        }
    }
    let mut t = DummyTransform;

    let input = "start\n- a\n- **b**\n- *c*\n\nend";
    let output = "start\na, BOLD b BOLD, ITALIC c ITALIC\nend";
    let res = transform_markdown_string(input.to_string(), &mut t);
    assert!(res.is_ok(), "Error on transformation: {res:?}");
    assert_eq!(res.unwrap(), output.to_string());
}
