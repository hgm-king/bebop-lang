pub mod parser;

pub type MarkdownText = Vec<MarkdownInline>;

use bytes::Bytes;

#[derive(Clone, Debug, PartialEq)]
pub enum Markdown {
    Heading(usize, MarkdownText),
    OrderedList(Vec<MarkdownText>),
    UnorderedList(Vec<MarkdownText>),
    Line(MarkdownText),
    Codeblock(Bytes, Bytes),
    Lisp(Bytes),
}

#[derive(Clone, Debug, PartialEq)]
pub enum MarkdownInline {
    Link(Bytes, Bytes),
    Image(Bytes, Bytes),
    InlineCode(Bytes),
    Bold(Bytes),
    Italic(Bytes),
    Plaintext(Bytes),
}
