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
