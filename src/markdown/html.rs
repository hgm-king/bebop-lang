use std::fmt;

use crate::markdown::{Markdown, MarkdownInline, MarkdownText};

pub struct HtmlString(String);

impl From<String> for HtmlString {
    fn from(md: String) -> Self {
        HtmlString(md)
    }
}

impl fmt::Display for HtmlString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Markdown> for HtmlString {
    fn from(md: Markdown) -> Self {
        match md {
            Markdown::Heading(level, text) => {
                format!("<h{}>{}</h{}>", level, HtmlString::from(text), level)
            }
            Markdown::Blockquote(text) => {
                format!("<blockquote>{}</blockquote>", HtmlString::from(text))
            }
            Markdown::UnorderedList(elements) => format!(
                "<ul>{}</ul>",
                elements
                    .into_iter()
                    .map(|element| format!("<li>{}</li>", HtmlString::from(element)))
                    .collect::<String>()
            ),
            Markdown::OrderedList(elements) => format!(
                "<ol>{}</ol>",
                elements
                    .into_iter()
                    .map(|element| format!("<li>{}</li>", HtmlString::from(element)))
                    .collect::<String>()
            ),
            Markdown::TaskList(elements) => format!(
                "<ul>{}</ul>",
                elements
                    .into_iter()
                    .map(|(checked, element)| if checked == true {
                        format!(
                            "<li><input type='checkbox' checked />{}</li>",
                            HtmlString::from(element)
                        )
                    } else {
                        format!(
                            "<li><input type='checkbox' />{}</li>",
                            HtmlString::from(element)
                        )
                    })
                    .collect::<String>()
            ),
            Markdown::Codeblock(lang, code) => {
                format!("<pre class=\"{}-snippet\">{}</pre>", lang, code)
            }
            Markdown::Line(text) => {
                if text.is_empty() {
                    String::from("<div></div>")
                } else {
                    format!("<p>{}</p>", HtmlString::from(text))
                }
            }
            Markdown::HorizontalRule => String::from("<hr />"),
            Markdown::Lisp(lisp) => format!("<pre>{}</pre>", lisp),
        }
        .into()
    }
}

impl FromIterator<HtmlString> for String {
    fn from_iter<I: IntoIterator<Item = HtmlString>>(iter: I) -> Self {
        let mut s = String::new();

        for i in iter {
            s = match i.into() {
                HtmlString(i) => format!("{}{}", s, i),
            };
        }

        s.into()
    }
}

impl FromIterator<MarkdownInline> for HtmlString {
    fn from_iter<I: IntoIterator<Item = MarkdownInline>>(iter: I) -> Self {
        let mut s = String::new();

        for i in iter {
            s = match i.into() {
                HtmlString(i) => format!("{}{}", s, i),
            };
        }

        s.into()
    }
}

impl From<MarkdownText> for HtmlString {
    fn from(md: MarkdownText) -> Self {
        md.into_iter().collect::<HtmlString>()
    }
}

impl From<MarkdownInline> for HtmlString {
    fn from(md: MarkdownInline) -> Self {
        match md {
            MarkdownInline::Bold(text) => {
                format!("<strong>{}</strong>", text)
            }
            MarkdownInline::Italic(text) => {
                format!("<em>{}</em>", text)
            }
            MarkdownInline::Strikethrough(text) => {
                format!("<s>{}</s>", text)
            }
            MarkdownInline::Link(text, href) => format!("<a href=\"{}\">{}</a>", href, text),
            MarkdownInline::ExternalLink(text, href) => {
                format!("<a target=\"_blank\" href=\"{}\">{}</a>", href, text)
            }
            MarkdownInline::Image(text, src) => format!("<img src=\"{}\" alt=\"{}\" />", src, text),
            MarkdownInline::InlineCode(text) => format!("<code>{}</code>", text),
            MarkdownInline::Color(text) => {
                format!("<span style=\"color: '{}'\">◼</span> {}", text, text)
            }
            MarkdownInline::Plaintext(text) => text.to_string(),
        }
        .into()
    }
}
