// TODO    Generate parametric tests

mod headers;
mod transform;

use pest::Parser;

use crate::MarkdownParser;
use crate::Rule;

#[test]
fn test_parse_markdown_input() {
    let input = "
# Je suis un titre h1
## Je suis un titre h2
[Je suis un texte de lien](je_suis_un_url)

[Je suis un texte de reference de lien][je_suis_une_ref_de_lien]

> Je suis une quote

**Je suis bold**

*Je suis italique*

Je suis un bout de `code`

```
    Je suis du code en block
```

### h3
#### h4
##### h5
###### h6

Bref
";

    let parsed = MarkdownParser::parse(Rule::file, input);
    println!("{parsed:?}");
    assert!(parsed.is_ok());
}
