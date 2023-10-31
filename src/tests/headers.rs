use crate::{MarkdownParser, Rule};
use pest::Parser;

#[test]
fn test_header_simple() {
    let mut input = "".to_string();
    for n in 1..7 {
        input += "#".repeat(n).as_str();
        input += format!(" h{n}\n").as_str();
    }
    let parsed = MarkdownParser::parse(Rule::file, &input);
    assert!(parsed.is_ok());
    let parsed = parsed.unwrap().next().unwrap();
    for line in parsed.into_inner() {
        let rule = line.as_rule();
        let mut inner = line.into_inner();
        match rule {
            Rule::h1 => assert_eq!(inner.next().unwrap().as_str(), "h1"),
            Rule::h2 => assert_eq!(inner.next().unwrap().as_str(), "h2"),
            Rule::h3 => assert_eq!(inner.next().unwrap().as_str(), "h3"),
            Rule::h4 => assert_eq!(inner.next().unwrap().as_str(), "h4"),
            Rule::h5 => assert_eq!(inner.next().unwrap().as_str(), "h5"),
            Rule::h6 => assert_eq!(inner.next().unwrap().as_str(), "h6"),
            _ => assert_eq!(inner.as_str(), ""),
        }
    }
}
