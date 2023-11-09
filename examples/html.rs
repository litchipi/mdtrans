use std::{collections::HashMap, path::PathBuf};

use mdtrans::{transform_markdown_string, MarkdownTransformer};

extern crate mdtrans;

#[derive(Default)]
pub struct Transformer {
    refs: HashMap<String, String>,
}

impl Transformer {
    fn sanitize_html(&self, text: String) -> String {
        text.replace('<', "&lt;").replace('>', "&gt;")
    }
}

impl MarkdownTransformer for Transformer {
    fn transform_text(&mut self, text: String) -> String {
        self.sanitize_html(text)
    }

    fn transform_quote(&mut self, text: String) -> String {
        format!("<div class=\"quote\">{text}</div>")
    }

    fn transform_image(
        &mut self,
        alt: String,
        url: String,
        _add_tags: std::collections::HashMap<String, String>,
    ) -> String {
        format!("<img src=\"{url}\" alt=\"{alt}\">")
    }

    fn transform_bold(&mut self, text: String) -> String {
        format!("<strong>{text}</strong>")
    }

    fn transform_italic(&mut self, text: String) -> String {
        format!("<em>{text}</em>")
    }

    fn transform_link(&mut self, text: String, url: String) -> String {
        format!("<a href=\"{url}\">{text}</a>")
    }

    fn transform_header(&mut self, level: usize, text: String) -> String {
        format!("<h{level}>{text}</h{level}>")
    }

    fn transform_inline_code(&mut self, text: String) -> String {
        format!("<code>{}</code>", self.sanitize_html(text))
    }

    fn transform_codeblock(&mut self, text: String) -> String {
        format!("<pre><code>{}</code></pre>", self.sanitize_html(text))
    }

    fn peek_refurl(&mut self, slug: String, url: String) {
        self.refs.insert(slug, url);
    }

    fn transform_reflink(&mut self, text: String, slug: String) -> String {
        let url = self.refs.get(&slug);
        assert!(url.is_some(), "Link reference {slug} not found");
        self.transform_link(text, url.unwrap().clone())
    }

    fn transform_refurl(&mut self, _slug: String, _url: String) -> String {
        "".to_string()
    }

    fn transform_list(&mut self, elements: Vec<String>) -> String {
        let mut buffer = "<ul>\n".to_string();
        buffer += elements.join("\n").as_str();
        buffer += "\n</ul>";
        buffer
    }

    fn transform_list_element(&mut self, element: String) -> String {
        format!("<li>{}</li>", self.sanitize_html(element))
    }

    fn transform_paragraph(&mut self, text: String) -> String {
        format!("<p>{text}</p>")
    }

    fn transform_vertical_space(&mut self) -> String {
        "<br/>".to_string()
    }
}

fn create_page(post: String) -> String {
    format!(
        "
        <!DOCTYPE html>
        <html lang=\"en\">
        <head>
            <meta charset=\"UTF-8\"> 
            <title>Post 1</title>
        </head>
        {post}
        </html>
    "
    )
}

fn main() {
    let mut transformer = Transformer::default();

    for file in std::fs::read_dir("./examples/data").unwrap() {
        let tstart = std::time::Instant::now();
        let post_file = file.unwrap().path();
        let fname = post_file
            .file_name()
            .unwrap()
            .to_os_string()
            .into_string()
            .unwrap();
        if !fname.ends_with(".md") {
            continue;
        }
        println!();
        let new_fname = fname.replace(".md", ".html");
        println!("{} -> {}", fname, new_fname);
        let post = std::fs::read_to_string(&post_file).unwrap();
        let res = transform_markdown_string(post, &mut transformer).unwrap();
        std::fs::write(PathBuf::from(new_fname), create_page(res)).unwrap();
        println!("Done in {:?}", tstart.elapsed());
    }
}
