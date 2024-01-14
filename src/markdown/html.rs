use crate::markdown::{Markdown, MarkdownInline, MarkdownText};

pub fn markdown_to_html(md: Markdown) -> String {
    match md {
        Markdown::Heading(level, text) => {
            format!("<h{}>{}</h{}>", level, text_to_html(text), level)
        }
        Markdown::Blockquote(text) => {
            format!("<blockquote>{}</blockquote>", text_to_html(text))
        },
        Markdown::UnorderedList(elements) => format!(
            "<ul>{}</ul>",
            elements
                .into_iter()
                .map(|element| format!("<li>{}</li>", text_to_html(element)))
                .collect::<String>()
        ),
        Markdown::OrderedList(elements) => format!(
            "<ol>{}</ol>",
            elements
                .into_iter()
                .map(|element| format!("<li>{}</li>", text_to_html(element)))
                .collect::<String>()
        ),
        Markdown::TaskList(elements) => format!(
            "<ul>{}</ul>",
            elements
                .into_iter()
                .map(|(checked, element)| if checked == true {
                    format!("<li><input type='checkbox' checked />{}</li>", text_to_html(element))
                } else {
                    format!("<li><input type='checkbox' />{}</li>", text_to_html(element))
                })
                .collect::<String>()
        ),
        Markdown::Codeblock(lang, code) => format!(
            "<pre class=\"{}-snippet\">{}</pre>",
            std::str::from_utf8(lang.as_bytes()).unwrap(),
            std::str::from_utf8(code.as_bytes()).unwrap()
        ),
        Markdown::Line(text) => {
            if text.is_empty() {
                String::from("<div></div>")
            } else {
                format!("<p>{}</p>", text_to_html(text))
            }
        }
        Markdown::HorizontalRule => String::from("<hr />"),
        Markdown::Lisp(lisp) => format!("<pre>{}</pre>", std::str::from_utf8(lisp.as_bytes()).unwrap()),
    }
}

fn text_to_html(md: MarkdownText) -> String {
    md.into_iter().map(inline_to_html).collect::<String>()
}

fn inline_to_html(md: MarkdownInline) -> String {
    match md {
        MarkdownInline::Bold(text) => {
            format!("<strong>{}</strong>", std::str::from_utf8(text.as_bytes()).unwrap())
        },
        MarkdownInline::Italic(text) => {
            format!("<em>{}</em>", std::str::from_utf8(text.as_bytes()).unwrap())
        },
        MarkdownInline::Strikethrough(text) => {
            format!("<s>{}</s>", std::str::from_utf8(text.as_bytes()).unwrap())
        },
        MarkdownInline::Link(text, href) => format!(
            "<a href=\"{}\">{}</a>",
            std::str::from_utf8(href.as_bytes()).unwrap(),
            std::str::from_utf8(text.as_bytes()).unwrap()
        ),
        MarkdownInline::ExternalLink(text, href) => format!(
            "<a target=\"_blank\" href=\"{}\">{}</a>",
            std::str::from_utf8(href.as_bytes()).unwrap(),
            std::str::from_utf8(text.as_bytes()).unwrap()
        ),
        MarkdownInline::Image(text, src) => format!(
            "<img src=\"{}\" alt=\"{}\" />",
            std::str::from_utf8(src.as_bytes()).unwrap(),
            std::str::from_utf8(text.as_bytes()).unwrap()
        ),
        MarkdownInline::InlineCode(text) => format!(
            "<code>{}</code>",
            std::str::from_utf8(text.as_bytes()).unwrap()
        ),
        MarkdownInline::Color(text) => format!(
            "<span style=\"color: '{}'\">◼</span> {}",
            std::str::from_utf8(text.as_bytes()).unwrap(),
            std::str::from_utf8(text.as_bytes()).unwrap()
        ),
        MarkdownInline::Plaintext(text) => {
            std::str::from_utf8(text.as_bytes()).unwrap().to_string()
        }
    }
}
