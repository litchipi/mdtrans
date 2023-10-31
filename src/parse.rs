// TODO    Rewrite
//    When parsing a block of text:
//    - Encounter a starting symbol triggers a loop
//        - Get all following chars into a buffer
//        - Then parse this buffer
//        - Then add the parsed nodes into the containing node
//
//    Function    add_char_to_buffer_while(big_buffer, local_buffer, |buffer: &String| { buffer.last() != "\n" }) (example)
//    then        let nodes = parse_buffer(local_buffer)
//                let container = MarkdownNode::Link(nodes, url); (example)
//    If the container is expected to contain specific data, parse it this way (metadata for example)

//    Something like:
//        - If got char '['
//            - Loop while we get char ']'    (if got backslash, ignore next ']')
//            - If we don't get it, treat it as raw text
//            - If we got it, search inside following chars for another delimiter
//                - If got '(', search for link url container
//                - If got '[', search for link reference container
//                - Else, treat as raw text

//    Put the helper functions first, test them
//    Then, add the parametric tests on the helper functions
//    Then add dummy implementations of all the symbols to parse
//    Then, add the parametric tests for each symbol to parse
//    Then implement with (parametric) test driven method

use std::collections::HashMap;

pub type MetadataType = HashMap<String, String>;
pub type MdNodes = Vec<Box<MarkdownNode>>;

#[derive(Debug, Clone)]
pub enum MarkdownNode {
    RawText(String),
    Text(MdNodes),
    Header(usize, MdNodes),
    Bold(MdNodes),
    Italic(MdNodes),
    Image(MdNodes, String),
    Strikethrough,

    Link(MdNodes, String),
    LinkReference(MdNodes, String),    // Points to a source
    LinkSource(String, String),
    // TODO    Tables
    // TODO    Code
    // TODO    Inline code
    // TODO    Quotes
}

impl MarkdownNode {
    pub fn recursive_detect(&mut self) {
    }
}

pub struct MarkdownParser {
    nodes: Vec<MarkdownNode>,
}

impl MarkdownParser {
    fn new() -> MarkdownParser {
        MarkdownParser {
            nodes: vec![],
        }
    }

    pub fn parse(md_text: String) -> Vec<MarkdownNode> {
        let mut parser = Self::new();
        let mut buffer = String::new();
        let mut buffer_now = String::new();
        for line in md_text.lines() {
            parser.feed_line(line.to_string() + "\n", &mut buffer, &mut buffer_now);
        }
        if !buffer.is_empty() {
            let node = MarkdownNode::RawText(buffer);
            parser.nodes.push(node);            
        }
        parser.nodes
    }

    pub fn feed_line(&mut self, line: String, buffer: &mut String, buffer_now: &mut String) {
        let line = line.trim_start();
        for c in line.chars() {
            buffer.push(c);
            if !c.is_whitespace() {
                buffer_now.push(c);
            }
            if let Some(mut nodes) = self.detect_node(buffer, buffer_now) {
                nodes.iter_mut().for_each(|node| node.recursive_detect());
                self.nodes.extend(nodes);
                *buffer = "".to_string();
                *buffer_now = "".to_string();
            }
        }
    }

    pub fn detect_node(&self, buffer: &String, buffer_now: &String) -> Option<Vec<MarkdownNode>> {
        if buffer.starts_with("#") && buffer.ends_with('\n') {
            if let Some(start) = buffer.find("# ") {
                let level = start + 1;
                let text = buffer.split_at(level+1).1.trim_end().to_string();
                return Some(vec![MarkdownNode::Header(level, wrap_mdnode(MarkdownNode::RawText(text)))]);
            }
        }
        if buffer.trim_start() == "---\n" {
           return Some(vec![MarkdownNode::Strikethrough]);
        }
        if buffer.ends_with(")") {
            if buffer_now.contains("](") {
                let text_end = buffer.find("](").unwrap();
                let url_start = text_end + 2;

                let mut nb_open = 0;
                let mut nb_close = 0;
                let mut last_open = 0;
                for (ind, c) in buffer[..text_end].chars().enumerate() {
                    if c == '[' {
                        nb_open += 1;
                        last_open = ind;
                    } else if c == ']' {
                        nb_close += 1;
                    }
                }
                if nb_close < nb_open {
                    let text_start = last_open + 1;
                    let text_before = buffer.split_at(last_open).0;
                    let node_before = MarkdownNode::RawText(text_before.to_string());
                    let text_node = wrap_mdnode(MarkdownNode::RawText(buffer[text_start..text_end].to_string()));
                    let url_end = buffer.len() - 1;
                    let node = MarkdownNode::Link(text_node, buffer[url_start..url_end].to_string());
                    return Some(vec![node_before, node]);
                }
            }
        }
        None
    }
}

fn wrap_mdnode(node: MarkdownNode) -> MdNodes {
    vec![Box::new(node)]
}

// TODO    Parametric tests

#[test]
fn test_header() {
    for exp_level in 1..7 {
        let text = "#".repeat(exp_level) + " Some header\n";
        let nodes = MarkdownParser::parse(text.to_string());
        let res = nodes.get(0).unwrap();
        if let MarkdownNode::Header(level, nodes) = res {
            assert_eq!(*level, exp_level);
            let node = nodes.get(0).unwrap();
            if let MarkdownNode::RawText(text) = node.as_ref() {
                assert_eq!(text, "Some header");
            } else {
                assert!(false);
            }
        } else {
            assert!(false);
        }
    }
}

#[test]
fn test_strikethrough() {
    let text = "  \t    ---\n";
    let nodes = MarkdownParser::parse(text.to_string());
    let res = nodes.get(0).unwrap();
    if let MarkdownNode::Strikethrough = res {
        assert!(true);
    } else {
        assert!(false);
    }
}

#[test]
fn test_link() {
    let text = "ablablabla\nallaezflk[text to link](url_of_the_link)zlkjazlkejfe\n";
    let nodes = MarkdownParser::parse(text.to_string());
    println!("{:?}", nodes);
    let res = nodes.get(0).unwrap();
    if let MarkdownNode::RawText(text) = res {
        assert_eq!(text, "ablablabla\nallaezflk");
    } else {
        assert!(false);
    }

    let res = nodes.get(1).unwrap();
    if let MarkdownNode::Link(text, url) = res {
        if let MarkdownNode::RawText(ref text) = **text.get(0).unwrap() {
            assert_eq!(text, "text to link");
        } else {
            assert!(false);
        }
        assert_eq!(url, "url_of_the_link");
    } else {
        assert!(false);
    }

    let text = "ablablabla\nallaezflk[text \nto link]\n(url_of_the_link\n)zlkjazlkejfe\n";
    let nodes = MarkdownParser::parse(text.to_string());
    println!("{:?}", nodes);
    let res = nodes.get(0).unwrap();
    if let MarkdownNode::RawText(text) = res {
        assert_eq!(text, "ablablabla\nallaezflk");
    } else {
        assert!(false);
    }

    let res = nodes.get(1).unwrap();
    if let MarkdownNode::Link(text, url) = res {
        if let MarkdownNode::RawText(ref text) = **text.get(0).unwrap() {
            assert_eq!(text, "text \nto link");
        } else {
            assert!(false);
        }
        assert_eq!(url, "url_of_the_link\n");
    } else {
        assert!(false);
    }
}
