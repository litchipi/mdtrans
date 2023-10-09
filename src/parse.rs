use std::collections::HashMap;

pub type MetadataType = HashMap<String, String>;
pub type MdNodes = Vec<Box<MarkdownNode>>;

#[derive(Clone)]
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
}

impl MarkdownNode {
    pub fn recursive_detect(&mut self) {
    }
}

enum ParsingAction {
    BackToParentNode,
    NewChild(MarkdownNode),
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
        for line in md_text.lines() {
            parser.feed_line(line.to_string() + "\n", &mut buffer);
        }
        parser.nodes
    }

    pub fn feed_line(&mut self, line: String, buffer: &mut String) {
        let line = line.trim_start();
        for c in line.chars() {
            buffer.push(c);
            if let Some(mut node) = self.detect_node(buffer) {
                node.recursive_detect();
                self.nodes.push(node);
                *buffer = "".to_string();
            }
        }
    }

    pub fn detect_node(&self, buffer: &String) -> Option<MarkdownNode> {
        if buffer.starts_with("#") && buffer.ends_with('\n') {
            if let Some(start) = buffer.find("# ") {
                let level = start + 1;
                let text = buffer.split_at(level+1).1.trim_end().to_string();
                return Some(MarkdownNode::Header(level, wrap_mdnode(MarkdownNode::RawText(text))));
            }
        }
        None
    }
}

fn wrap_mdnode(node: MarkdownNode) -> MdNodes {
    vec![Box::new(node)]
}

#[test]
fn test_detect_header() {
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
