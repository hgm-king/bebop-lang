pub mod html;
pub mod parser;

use bytes::Bytes;

#[derive(Debug, PartialEq)]
pub enum Markdown {
    Heading(usize, MarkdownText),
    OrderedList(Vec<MarkdownText>),
    UnorderedList(Vec<MarkdownText>),
    Line(MarkdownText),
    Codeblock(Bytes, Bytes),
    Lisp(Bytes),
}

pub type MarkdownText = Vec<MarkdownInline>;

#[derive(Debug, PartialEq)]
pub enum MarkdownInline {
    Link(Bytes, Bytes),
    Image(Bytes, Bytes),
    InlineCode(Bytes),
    Bold(Bytes),
    Italic(Bytes),
    Plaintext(Bytes),
}

pub fn markdown_to_html(md: Bytes) -> Result<String, String> {
    let (_, md) = parser::parse_markdown(nombytes::NomBytes::new(md)).map_err(|e| {
        println!("{:?}", e);
        String::from("Not valid md")
    })?;

    Ok(md.into_iter().map(|md| html::markdown_to_html(md)).collect())
}