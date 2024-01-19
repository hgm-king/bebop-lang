use std::fmt;

use crate::markdown::{Markdown, MarkdownInline, MarkdownText};

pub struct LispString(String);

impl From<String> for LispString {
    fn from(md: String) -> Self {
        LispString(md)
    }
}

impl fmt::Display for LispString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Markdown> for LispString {
    fn from(md: Markdown) -> Self {
        match md {
            Markdown::Heading(level, text) => {
                format!("(h{} (concat {}))\n", level, LispString::from(text))
            }
            Markdown::Blockquote(text) => {
                format!("(blockquote (concat {}))\n", LispString::from(text))
            }
            Markdown::UnorderedList(elements) => format!(
                "(ul\n(concat {}))\n",
                elements
                    .into_iter()
                    .map(|element| format!("(li (concat {}))\n", LispString::from(element)))
                    .collect::<String>()
            ),
            Markdown::OrderedList(elements) => format!(
                "(ol\n(concat {}))\n",
                elements
                    .into_iter()
                    .map(|element| format!("\t(li (concat {}))\n", LispString::from(element)))
                    .collect::<String>()
            ),
            Markdown::TaskList(elements) => format!(
                "(tasks\n(concat {}))\n",
                elements
                    .into_iter()
                    .map(|(checked, element)| if checked == true {
                        format!("\t(li (concat checked {}))\n", LispString::from(element))
                    } else {
                        format!("\t(li (concat unchecked {}))\n", LispString::from(element))
                    })
                    .collect::<String>()
            ),
            Markdown::Codeblock(_, code) => format!("(pre \"{}\")\n", code),
            Markdown::Line(text) => {
                if text.is_empty() {
                    String::from("(empty)\n")
                } else {
                    format!("(p (concat {}))\n", LispString::from(text))
                }
            }
            Markdown::HorizontalRule => String::from("hr\n"),
            Markdown::Lisp(lisp) => format!("{} ", lisp),
        }
        .into()
    }
}

impl FromIterator<LispString> for String {
    fn from_iter<I: IntoIterator<Item = LispString>>(iter: I) -> Self {
        let mut s = String::new();

        for i in iter {
            s = match i.into() {
                LispString(i) => format!("{}{}", s, i),
            };
        }

        s.into()
    }
}

impl FromIterator<MarkdownInline> for LispString {
    fn from_iter<I: IntoIterator<Item = MarkdownInline>>(iter: I) -> Self {
        let mut s = String::new();

        for i in iter {
            s = match i.into() {
                LispString(i) => format!("{}{}", s, i),
            };
        }

        s.into()
    }
}

impl From<MarkdownText> for LispString {
    fn from(md: MarkdownText) -> Self {
        md.into_iter().collect::<LispString>()
    }
}

impl From<MarkdownInline> for LispString {
    fn from(md: MarkdownInline) -> Self {
        match md {
            MarkdownInline::Bold(text) => {
                format!("(strong \"{}\") ", text)
            }
            MarkdownInline::Italic(text) => {
                format!("(em \"{}\") ", text)
            }
            MarkdownInline::Link(text, href) => format!("(a \"{}\" \"{}\") ", href, text),
            MarkdownInline::ExternalLink(text, href) => {
                format!("(a-out \"{}\" \"{}\") ", href, text)
            }
            MarkdownInline::Image(text, src) => format!("(img \"{}\" \"{}\") ", src, text),
            MarkdownInline::Strikethrough(text) => format!("(strike \"{}\") ", text),
            MarkdownInline::InlineCode(text) => format!("(code \"{}\") ", text),
            MarkdownInline::Color(text) => format!("(color \"{}\") ", text),
            MarkdownInline::Plaintext(text) => format!("\"{}\" ", text.to_string()),
        }
        .into()
    }
}
