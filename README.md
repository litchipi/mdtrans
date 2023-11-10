# Mdtrans
Markdown parser and transformer, in Rust, using Pest

This is not the fastest parser / transformer, but it's built in order to provide the most flexibility to your needs.

## Usage

``` rust
#[derive(Default)]
pub struct MyOwnTransformer {
   image_count: usize,
   image_trans: usize,
}

impl MarkdownTransformer for MyOwnTransformer {
    fn peek_image(&mut self, alt: String, url: String, add_tags: HashMap<String, String>) {
        self.image_count_total += 1;
    }
    fn tranform_image(&mut self, alt: String, url: String, add_tags: HashMap<String, String>) -> String {
        self.image_trans += 1;
        format!("Image {}/{} <img alt=\"{alt}\" href=\"{url}\">", self.image_trans, self.image_count)
    }
}

fn main() {
    let trans = MyOwnTransformer::default();
    let input = " ... ".to_string();
    let output = transform_markdown_string(input, &mut trans).unwrap();
    println!("{output}");
}
```
Inside the `transform_markdown_string` function, the transformer will perform all `peek` functions before.  
This means that in this code we first count the total number of images, and then transform each one of them.  
The result will be something like:
``` html
Image 1/2 <img alt="toto" href="url">
Image 2/2 <img alt="tutu" href="url">
```

For an example of Markdown-to-HTML implementation, see [this file](https://github.com/litchipi/mdtrans/blob/main/examples/html.rs#L18)  
For the definition of the trait itself, see [this file](https://github.com/litchipi/mdtrans/blob/main/src/transform.rs#L10)

## Contribute
This is a hobby side-project, but you can contribute if you feel like it !  
- Contributions on the [pest grammar file](https://github.com/litchipi/mdtrans/blob/main/markdown.pest) are appreciated as I'm really not an expert in it
- You can open an issue if you find some Markdown inputs that are not supported (or not well) by this engine, or causes bugs.
