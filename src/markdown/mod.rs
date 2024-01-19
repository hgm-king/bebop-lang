use self::{html::HtmlString, lisp::LispString};

pub mod html;
pub mod lisp;
pub mod parser;

#[derive(Debug, PartialEq)]
pub enum Markdown {
    Heading(usize, MarkdownText),
    OrderedList(Vec<MarkdownText>),
    UnorderedList(Vec<MarkdownText>),
    TaskList(Vec<(bool, MarkdownText)>),
    Line(MarkdownText),
    Codeblock(String, String),
    Blockquote(MarkdownText),
    HorizontalRule,
    Lisp(String),
}

pub type MarkdownText = Vec<MarkdownInline>;

#[derive(Debug, PartialEq)]
pub enum MarkdownInline {
    Link(String, String),
    ExternalLink(String, String),
    Image(String, String),
    InlineCode(String),
    Bold(String),
    Italic(String),
    Plaintext(String),
    Strikethrough(String),
    Color(String),
}

pub fn markdown_to_html(md: &str) -> Result<String, String> {
    let (_, md) = parser::parse_markdown(md).map_err(|e| {
        println!("{:?}", e);
        String::from("Not valid md")
    })?;

    Ok(md.into_iter().map(|md| HtmlString::from(md)).collect::<String>())
}

pub fn markdown_to_lisp(md: &str) -> Result<String, String> {
    let (_, md) = parser::parse_markdown(md).map_err(|e| {
        println!("{:?}", e);
        String::from("Not valid md")
    })?;

    Ok(md.into_iter().map(|md| LispString::from(md)).collect::<String>())
}