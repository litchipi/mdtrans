use crate::{transform_markdown_string, MarkdownTransformer};

#[test]
fn test_trait_impl() {
    pub struct DummyTransform;
    impl MarkdownTransformer for DummyTransform {
        fn transform_comment(&mut self, text: String) -> String {
            format!("COMMENT {text} COMMENT")
        }
    }
    let mut t = DummyTransform;
    let res = transform_markdown_string("<!-- comment -->".to_string(), &mut t);
    assert!(res.is_ok(), "Error on transformation: {res:?}");
    assert_eq!(res.unwrap(), "COMMENT comment COMMENT");

    let res = transform_markdown_string("<!--  comment\nblock\n -->".to_string(), &mut t);
    assert!(res.is_ok(), "Error on transformation: {res:?}");
    assert_eq!(res.unwrap(), "COMMENT comment\nblock COMMENT");
}

#[test]
fn test_empty_rich_text() {
    pub struct DummyTransform;
    impl MarkdownTransformer for DummyTransform {
        fn transform_link(&mut self, text: String, url: String) -> String {
            format!("{text}: {url}")
        }
    }
    let mut t = DummyTransform;
    let res = transform_markdown_string("[](url)".to_string(), &mut t);
    assert!(res.is_ok(), "Error on transformation: {res:?}");
    assert_eq!(res.unwrap(), ": url");
}

#[test]
fn test_transform_empty() {
    pub struct DummyTransform;
    impl MarkdownTransformer for DummyTransform {}
    let mut t = DummyTransform;
    let res = transform_markdown_string("".to_string(), &mut t);
    assert!(res.is_ok(), "Error on transformation: {res:?}");
}

#[test]
fn test_transform_string() {
    pub struct DummyTransform;
    impl MarkdownTransformer for DummyTransform {
        fn transform_text(&mut self, text: String) -> String {
            text
        }
    }
    let mut t = DummyTransform;
    let res = transform_markdown_string("elzkaj".to_string(), &mut t);
    assert!(res.is_ok(), "Error on transformation: {res:?}");
    assert_eq!(res.unwrap(), "elzkaj".to_string());

    let res = transform_markdown_string("a\nb\nc".to_string(), &mut t);
    assert!(res.is_ok(), "Error on transformation: {res:?}");
    assert_eq!(res.unwrap(), "a b c".to_string());
}

#[test]
fn test_transform_header() {
    pub struct DummyTransform;
    impl MarkdownTransformer for DummyTransform {
        fn transform_header(&mut self, level: usize, text: String) -> String {
            format!("h{level}: {text}")
        }
    }
    let mut t = DummyTransform;

    for level in 1..7 {
        let res = transform_markdown_string(format!("{} header", "#".repeat(level)), &mut t);
        assert!(res.is_ok(), "Error on transformation: {res:?}");
        assert_eq!(res.unwrap(), format!("h{level}: header"));
    }

    let input = "## Some `code` here **bold** ok";
    let output = "h2: Some code here bold ok";
    let res = transform_markdown_string(input.to_string(), &mut t);
    assert!(res.is_ok(), "Error on transformation: {res:?}");
    assert_eq!(res.unwrap(), output);
}

#[test]
fn test_transform_italic() {
    pub struct DummyTransform;
    impl MarkdownTransformer for DummyTransform {
        fn transform_italic(&mut self, text: String) -> String {
            format!("ITALIC {text} ITALIC")
        }
    }
    let mut t = DummyTransform;

    let res = transform_markdown_string("*toto*".to_string(), &mut t);
    assert!(res.is_ok(), "Error on transformation: {res:?}");
    assert_eq!(res.unwrap(), "ITALIC toto ITALIC".to_string());
}

#[test]
fn test_transform_strike() {
    pub struct DummyTransform;
    impl MarkdownTransformer for DummyTransform {
        fn transform_strikethrough(&mut self, text: String) -> String {
            format!("STRIKE {text} STRIKE")
        }
    }
    let mut t = DummyTransform;

    let res = transform_markdown_string("~~toto~~".to_string(), &mut t);
    assert!(res.is_ok(), "Error on transformation: {res:?}");
    assert_eq!(res.unwrap(), "STRIKE toto STRIKE".to_string());

    let res = transform_markdown_string("--toto--".to_string(), &mut t);
    assert!(res.is_ok(), "Error on transformation: {res:?}");
    assert_eq!(res.unwrap(), "STRIKE toto STRIKE".to_string());
}

#[test]
fn test_transform_bold() {
    pub struct DummyTransform;
    impl MarkdownTransformer for DummyTransform {
        fn transform_bold(&mut self, text: String) -> String {
            format!("BOLD {text} BOLD")
        }

        fn transform_italic(&mut self, text: String) -> String {
            format!("ITALIC {text} ITALIC")
        }
    }
    let mut t = DummyTransform;

    let input = "**toto**";
    let output = "BOLD toto BOLD";
    let res = transform_markdown_string(input.to_string(), &mut t);
    assert!(res.is_ok(), "Error on transformation: {res:?}");
    assert_eq!(res.unwrap(), output.to_string());

    let input = "**toto *italic* tutu**";
    let output = "BOLD toto ITALIC italic ITALIC tutu BOLD";
    let res = transform_markdown_string(input.to_string(), &mut t);
    assert!(res.is_ok(), "Error on transformation: {res:?}");
    assert_eq!(res.unwrap(), output.to_string());
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
        fn transform_codeblock(&mut self, lang: Option<String>, text: String) -> String {
            let mut buffer = "\nCODEBLOCK".to_string();
            if let Some(l) = lang {
                buffer += format!(" {l}").as_str();
            }
            buffer += format!("\n{text}\nCODEBLOCK\n").as_str();
            buffer
        }
    }
    let mut t = DummyTransform;

    let input = "start\n```\nsome\ncode\n```\nend";
    let output = "start\nCODEBLOCK\nsome\ncode\nCODEBLOCK\nend";
    let res = transform_markdown_string(input.to_string(), &mut t);
    assert!(res.is_ok(), "Error on transformation: {res:?}");
    assert_eq!(res.unwrap(), output.to_string());

    let input = "start\n``` lang\nsome\ncode\n```\nend";
    let output = "start\nCODEBLOCK lang\nsome\ncode\nCODEBLOCK\nend";
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
            "\n=== HORIZ SEPARATOR ===\n".to_string()
        }
    }
    let mut t = DummyTransform;

    let input = "start\n\n---\nend";
    let output = "start\n=== HORIZ SEPARATOR ===\nend";
    let res = transform_markdown_string(input.to_string(), &mut t);
    assert!(res.is_ok(), "Error on transformation: {res:?}");
    assert_eq!(res.unwrap(), output.to_string());
}

#[test]
fn test_transform_list() {
    pub struct DummyTransform;
    impl MarkdownTransformer for DummyTransform {
        fn transform_list_element(&mut self, element: String) -> String {
            element
        }

        fn transform_list(&mut self, elements: Vec<String>) -> String {
            format!("\n{}\n", elements.join(", "))
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

#[test]
fn test_transform_image() {
    pub struct DummyTransform;
    impl MarkdownTransformer for DummyTransform {
        fn transform_image(
            &mut self,
            alt: String,
            url: String,
            add_tags: std::collections::HashMap<String, String>,
        ) -> String {
            let mut upper = false;
            if let Some(t) = add_tags.get("upper") {
                if t == "true" {
                    upper = true;
                }
            }
            format!(
                "{} -> {}",
                if upper { alt.to_uppercase() } else { alt },
                if upper { url.to_uppercase() } else { url }
            )
        }
    }
    let mut t = DummyTransform;

    let input = "start\n![image alt](url)\nend";
    let output = "start image alt -> url end";
    let res = transform_markdown_string(input.to_string(), &mut t);
    assert!(res.is_ok(), "Error on transformation: {res:?}");
    assert_eq!(res.unwrap(), output.to_string());

    let input = "start\n![image alt](url)[a: b, c:   d, upper: true, d  : e]\nend";
    let output = "start IMAGE ALT -> URL end";
    let res = transform_markdown_string(input.to_string(), &mut t);
    assert!(res.is_ok(), "Error on transformation: {res:?}");
    assert_eq!(res.unwrap(), output.to_string());
}
