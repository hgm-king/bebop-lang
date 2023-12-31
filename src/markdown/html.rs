use crate::markdown::{Markdown, MarkdownInline, MarkdownText};

pub fn markdown_to_html(md: Markdown) -> String {
    match md {
        Markdown::Heading(level, text) => {
            format!("<h{}>{}</h{}>", level, text_to_html(text), level)
        }
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
        Markdown::Codeblock(lang, code) => format!(
            "<pre><code class=\"{}-snippet\">{}</code></pre>",
            std::str::from_utf8(lang.as_bytes()).unwrap(),
            std::str::from_utf8(code.as_bytes()).unwrap()
        ),
        Markdown::Line(text) => {
            if text.is_empty() {
                String::from("<hr />")
            } else {
                format!("<p>{}</p>", text_to_html(text))
            }
        }
        Markdown::Lisp(lisp) => format!("<p>{}</p>", std::str::from_utf8(lisp.as_bytes()).unwrap()),
    }
}

fn text_to_html(md: MarkdownText) -> String {
    md.into_iter().map(inline_to_html).collect::<String>()
}

fn inline_to_html(md: MarkdownInline) -> String {
    match md {
        MarkdownInline::Bold(text) => {
            format!("<b>{}</b>", std::str::from_utf8(text.as_bytes()).unwrap())
        }
        MarkdownInline::Italic(text) => {
            format!("<i>{}</i>", std::str::from_utf8(text.as_bytes()).unwrap())
        }
        MarkdownInline::Link(text, href) => format!(
            "<a href=\"{}\">{}</a>",
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
        MarkdownInline::Plaintext(text) => {
            std::str::from_utf8(text.as_bytes()).unwrap().to_string()
        }
    }
}
