use crate::markdown::{Markdown, MarkdownInline, MarkdownText};

pub fn markdown_to_lisp(md: Markdown) -> String {
    match md {
        Markdown::Heading(level, text) => {
            format!("(h{} (concat {}))\n", level, text_to_lisp(text))
        },
        Markdown::Blockquote(text) => {
            format!("(blockquote (concat {}))\n", text_to_lisp(text))
        },
        Markdown::UnorderedList(elements) => format!(
            "(ul\n(concat {}))\n",
            elements
                .into_iter()
                .map(|element| format!("(li (concat {}))\n", text_to_lisp(element)))
                .collect::<String>()
        ),
        Markdown::OrderedList(elements) => format!(
            "(ol\n(concat {}))\n",
            elements
                .into_iter()
                .map(|element| format!("\t(li (concat {}))\n", text_to_lisp(element)))
                .collect::<String>()
        ),
        Markdown::TaskList(elements) => format!(
            "(ul\n(concat {}))\n",
            elements
                .into_iter()
                .map(|(checked, element)| if checked == true {
                    format!("\t(li (concat checked {}))\n", text_to_lisp(element))
                } else {
                    format!("\t(li (concat unchecked {}))\n", text_to_lisp(element))
                })
                .collect::<String>()
        ),
        Markdown::Codeblock(_, code) => format!(
            "(pre (code \"{}\"))\n",
            std::str::from_utf8(code.as_bytes()).unwrap()
        ),
        Markdown::Line(text) => {
            if text.is_empty() {
                String::from("empty\n")
            } else {
                format!("(p (concat {}))\n", text_to_lisp(text))
            }
        }
        Markdown::HorizontalRule => String::from("hr\n"),
        Markdown::Lisp(lisp) => format!("{} ", std::str::from_utf8(lisp.as_bytes()).unwrap()),
    }
}

fn text_to_lisp(md: MarkdownText) -> String {
    md.into_iter().map(inline_to_lisp).collect::<String>()
}

fn inline_to_lisp(md: MarkdownInline) -> String {
    match md {
        MarkdownInline::Bold(text) => {
            format!(
                "(strong \"{}\") ",
                std::str::from_utf8(text.as_bytes()).unwrap()
            )
        }
        MarkdownInline::Italic(text) => {
            format!(
                "(em \"{}\") ",
                std::str::from_utf8(text.as_bytes()).unwrap()
            )
        }
        MarkdownInline::Link(text, href) => format!(
            "(a \"{}\" \"{}\") ",
            std::str::from_utf8(href.as_bytes()).unwrap(),
            std::str::from_utf8(text.as_bytes()).unwrap()
        ),
        MarkdownInline::ExternalLink(text, href) => format!(
            "(a-out \"{}\" \"{}\") ",
            std::str::from_utf8(href.as_bytes()).unwrap(),
            std::str::from_utf8(text.as_bytes()).unwrap()
        ),
        MarkdownInline::Image(text, src) => format!(
            "(img \"{}\" \"{}\") ",
            std::str::from_utf8(src.as_bytes()).unwrap(),
            std::str::from_utf8(text.as_bytes()).unwrap()
        ),
        MarkdownInline::Strikethrough(text) => format!(
            "(strike \"{}\") ",
            std::str::from_utf8(text.as_bytes()).unwrap()
        ),
        MarkdownInline::InlineCode(text) => format!(
            "(code \"{}\") ",
            std::str::from_utf8(text.as_bytes()).unwrap()
        ),
        MarkdownInline::Color(text) => format!(
            "(color \"{}\") ",
            std::str::from_utf8(text.as_bytes()).unwrap()
        ),
        MarkdownInline::Plaintext(text) => format!(
            "\"{}\" ",
            std::str::from_utf8(text.as_bytes()).unwrap().to_string()
        ),
    }
}
