pub mod html;
pub mod parser;

#[derive(Debug, PartialEq)]
pub enum Markdown {
    Heading(usize, MarkdownText),
    OrderedList(Vec<MarkdownText>),
    UnorderedList(Vec<MarkdownText>),
    Line(MarkdownText),
    Codeblock(String, String),
    Lisp(String),
}

pub type MarkdownText = Vec<MarkdownInline>;

#[derive(Debug, PartialEq)]
pub enum MarkdownInline {
    Link(String, String),
    Image(String, String),
    InlineCode(String),
    Bold(String),
    Italic(String),
    Plaintext(String),
}

pub fn markdown_to_html(md: &str) -> Result<String, String> {
    let (_, md) = parser::parse_markdown(md).map_err(|e| {
        println!("{:?}", e);
        String::from("Not valid md")
    })?;

    Ok(md
        .into_iter()
        .map(html::markdown_to_html)
        .collect())
}
