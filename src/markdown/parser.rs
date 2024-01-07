use crate::markdown::Markdown;
use crate::markdown::MarkdownInline;
use crate::markdown::MarkdownText;

use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take, take_while1},
    character::{is_digit, is_newline},
    combinator::{eof, map, peek},
    error::{Error, ErrorKind},
    multi::{many0, many1, many_till},
    sequence::{delimited, pair, preceded, terminated, tuple},
    Err as NomErr, IResult,
};

pub fn parse_markdown(i: &str) -> IResult<&str, Vec<Markdown>> {
    many1(alt((
        map(parse_header, |e| Markdown::Heading(e.0, e.1)),
        map(parse_unordered_list, Markdown::UnorderedList),
        map(parse_ordered_list, Markdown::OrderedList),
        map(parse_code_block, |e| Markdown::Codeblock(e.0, e.1)),
        map(parse_lisp, |e| Markdown::Lisp(e)),
        map(parse_markdown_text, Markdown::Line),
        map(parse_markdown_inline, |e| Markdown::Line(vec![e])),
    )))(i)
}

// **([^*][^*])+**
fn parse_boldtext(i: &str) -> IResult<&str, MarkdownInline> {
    map(delimited(tag("**"), is_not("**"), tag("**")), |b: &str| {
        MarkdownInline::Bold(b.to_string())
    })(i)
}

// *[^*]+*
fn parse_italics(i: &str) -> IResult<&str, MarkdownInline> {
    map(delimited(tag("*"), is_not("*"), tag("*")), |b: &str| {
        MarkdownInline::Italic(b.to_string())
    })(i)
}

// `[^`]+`
fn parse_inline_code(i: &str) -> IResult<&str, MarkdownInline> {
    map(delimited(tag("`"), is_not("`"), tag("`")), |b: &str| {
        MarkdownInline::InlineCode(b.to_string())
    })(i)
}

// \[[^\]]+\]\([^\)]\)
fn parse_link(i: &str) -> IResult<&str, MarkdownInline> {
    map(
        pair(
            delimited(tag("["), is_not("]"), tag("]")),
            delimited(tag("("), is_not(")"), tag(")")),
        ),
        |(b, c): (&str, &str)| MarkdownInline::Link(b.to_string(), c.to_string()),
    )(i)
}

// !\[[^\]]+\]\([^\)]\)
fn parse_image(i: &str) -> IResult<&str, MarkdownInline> {
    map(
        pair(
            delimited(tag("!["), is_not("]"), tag("]")),
            delimited(tag("("), is_not(")"), tag(")")),
        ),
        |(b, c): (&str, &str)| MarkdownInline::Image(b.to_string(), c.to_string()),
    )(i)
}

// // we want to match many things that are not any of our special tags
// // but since we have no tools available to match and consume in the negative case (without regex)
// // we need to match against our tags, then consume one char
// // we repeat this until we run into one of our special characters
// // then we join our array of characters into a &str
fn parse_plaintext(i: &str) -> IResult<&str, MarkdownInline> {
    let (i, (vec, _)) = many_till(
        take(1u8),
        alt((peek(alt((
            parse_boldtext,
            parse_italics,
            parse_inline_code,
            parse_image,
            parse_link,
            map(alt((tag("\r\n"), tag("\n"))), |t: &str| {
                MarkdownInline::Plaintext(t.to_string())
            }),
            map(eof, |t: &str| MarkdownInline::Plaintext(t.to_string())),
        ))),)),
    )(i)?;

    if vec.is_empty() {
        Err(NomErr::Error(Error {
            input: i,
            code: ErrorKind::Not,
        }))
    } else {
        Ok((
            i,
            MarkdownInline::Plaintext(vec.into_iter().map(|e| e.to_string()).collect::<String>()),
        ))
    }
}

fn parse_markdown_inline(i: &str) -> IResult<&str, MarkdownInline> {
    alt((
        parse_italics,
        parse_inline_code,
        parse_boldtext,
        parse_image,
        parse_link,
        parse_plaintext,
    ))(i)
}

fn parse_markdown_text(i: &str) -> IResult<&str, MarkdownText> {
    terminated(many0(parse_markdown_inline), alt((tag("\r\n"), tag("\n"))))(i)
}

// #*
fn parse_header_tag(i: &str) -> IResult<&str, usize> {
    map(
        terminated(take_while1(|c| c == '#'), tag(" ")),
        |s: &str| s.to_string().len(),
    )(i)
}

// this combines a tuple of the header tag and the rest of the line
fn parse_header(i: &str) -> IResult<&str, (usize, MarkdownText)> {
    tuple((parse_header_tag, parse_markdown_text))(i)
}

fn parse_unordered_list_tag(i: &str) -> IResult<&str, &str> {
    terminated(tag("-"), tag(" "))(i)
}

fn parse_unordered_list_element(i: &str) -> IResult<&str, MarkdownText> {
    preceded(parse_unordered_list_tag, parse_markdown_text)(i)
}

fn parse_unordered_list(i: &str) -> IResult<&str, Vec<MarkdownText>> {
    many1(parse_unordered_list_element)(i)
}

fn parse_ordered_list_tag(i: &str) -> IResult<&str, &str> {
    terminated(
        terminated(take_while1(|d| is_digit(d as u8)), tag(".")),
        tag(" "),
    )(i)
}

fn parse_ordered_list_element(i: &str) -> IResult<&str, MarkdownText> {
    preceded(parse_ordered_list_tag, parse_markdown_text)(i)
}

fn parse_ordered_list(i: &str) -> IResult<&str, Vec<MarkdownText>> {
    many1(parse_ordered_list_element)(i)
}

fn parse_code_block(i: &str) -> IResult<&str, (String, String)> {
    pair(parse_code_block_lang, parse_code_block_body)(i)
}

fn parse_code_block_body(i: &str) -> IResult<&str, String> {
    map(
        delimited(
            alt((tag("\r\n"), tag("\n"))),
            is_not("```"),
            pair(tag("```"), alt((eof, alt((tag("\r\n"), tag("\n")))))),
        ),
        |s: &str| s.to_string(),
    )(i)
}

fn parse_code_block_lang(i: &str) -> IResult<&str, String> {
    alt((
        preceded(
            tag("```"),
            map(take_while1(|c| !is_newline(c as u8)), |b: &str| {
                b.to_string()
            }),
        ),
        map(tag("```"), |_| String::from("__UNKNOWN__")),
    ))(i)
}

fn parse_lisp(i: &str) -> IResult<&str, String> {
    map(delimited(tag("|"), is_not("|"), tag("|")), |s: &str| {
        s.to_string()
    })(i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::{error::Error, error::ErrorKind, Err as NomErr};

    #[test]
    fn test_parse_italics() {
        assert_eq!(
            parse_italics("*here is italic*"),
            Ok(((""), MarkdownInline::Italic(String::from("here is italic"))))
        );
        assert_eq!(
            parse_italics("*here is italic*\n"),
            Ok((
                ("\n"),
                MarkdownInline::Italic(String::from("here is italic"))
            ))
        );
        assert!(parse_italics("*here is italic").is_err());
        assert!(parse_italics("here is italic*").is_err());
        assert!(parse_italics("here is italic").is_err());
        assert!(parse_italics("*").is_err());
        assert!(parse_italics("**").is_err());
        assert!(parse_italics("").is_err());
        assert!(parse_italics("**we are doing bold**").is_err());
    }

    #[test]
    fn test_parse_boldtext() {
        assert_eq!(
            parse_boldtext("**here is bold**"),
            Ok(((""), MarkdownInline::Bold(String::from("here is bold"))))
        );
        assert_eq!(
            parse_boldtext("**here is bold**\n"),
            Ok((("\n"), MarkdownInline::Bold(String::from("here is bold"))))
        );
        assert!(parse_boldtext("**here is bold").is_err());
        assert!(parse_boldtext("here is bold**").is_err());
        assert!(parse_boldtext("here is bold").is_err());
        assert!(parse_boldtext("****").is_err());
        assert!(parse_boldtext("**").is_err());
        assert!(parse_boldtext("*").is_err());
        assert!(parse_boldtext("").is_err());
        assert!(parse_boldtext("*this is italic*").is_err());
    }

    #[test]
    fn test_parse_inline_code() {
        assert_eq!(
            parse_inline_code("`here is bold`\n"),
            Ok((
                ("\n"),
                MarkdownInline::InlineCode(String::from("here is bold"))
            ))
        );
        assert!(parse_inline_code("`here is code").is_err());
        assert!(parse_inline_code("here is code`").is_err());
        assert!(parse_inline_code("``").is_err());
        assert!(parse_inline_code("`").is_err());
        assert!(parse_inline_code("").is_err());
    }

    #[test]
    fn test_parse_link() {
        assert_eq!(
            parse_link("[title](https://www.example.com)"),
            Ok((
                (""),
                MarkdownInline::Link(
                    String::from("title"),
                    String::from("https://www.example.com")
                )
            ))
        );
        assert!(parse_link("[title](whatever").is_err());
    }

    #[test]
    fn test_parse_image() {
        assert_eq!(
            parse_image("![alt text](image.jpg)"),
            Ok((
                (""),
                MarkdownInline::Image(String::from("alt text"), String::from("image.jpg"))
            ))
        );
        assert!(parse_image("[title](whatever").is_err());
    }

    #[test]
    fn test_parse_plaintext() {
        assert_eq!(
            parse_plaintext("1234567890"),
            Ok(((""), MarkdownInline::Plaintext(String::from("1234567890"))))
        );
        assert_eq!(
            parse_plaintext("oh my gosh!"),
            Ok(((""), MarkdownInline::Plaintext(String::from("oh my gosh!"))))
        );
        assert_eq!(
            parse_plaintext("oh my gosh!["),
            Ok((
                (""),
                MarkdownInline::Plaintext(String::from("oh my gosh!["))
            ))
        );
        assert_eq!(
            parse_plaintext("oh my gosh!*"),
            Ok((
                (""),
                MarkdownInline::Plaintext(String::from("oh my gosh!*"))
            ))
        );
        assert_eq!(
            parse_plaintext("*bold babey bold*"),
            Err(NomErr::Error(Error {
                input: ("*bold babey bold*"),
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_plaintext("[link babey](and then somewhat)"),
            Err(NomErr::Error(Error {
                input: ("[link babey](and then somewhat)"),
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_plaintext("`codeblock for bums`"),
            Err(NomErr::Error(Error {
                input: ("`codeblock for bums`"),
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_plaintext("![ but wait theres more](jk)"),
            Err(NomErr::Error(Error {
                input: ("![ but wait theres more](jk)"),
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_plaintext("here is plaintext"),
            Ok((
                (""),
                MarkdownInline::Plaintext(String::from("here is plaintext"))
            ))
        );
        assert_eq!(
            parse_plaintext("here is plaintext!"),
            Ok((
                (""),
                MarkdownInline::Plaintext(String::from("here is plaintext!"))
            ))
        );
        assert_eq!(
            parse_plaintext("here is plaintext![image starting"),
            Ok((
                (""),
                MarkdownInline::Plaintext(String::from("here is plaintext![image starting"))
            ))
        );
        assert_eq!(
            parse_plaintext("here is plaintext\n"),
            Ok((
                ("\n"),
                MarkdownInline::Plaintext(String::from("here is plaintext"))
            ))
        );
        assert_eq!(
            parse_plaintext("here is plaintext\nand the next line"),
            Ok((
                ("\nand the next line"),
                MarkdownInline::Plaintext(String::from("here is plaintext"))
            ))
        );
        assert_eq!(
            parse_plaintext("*here is italic*"),
            Err(NomErr::Error(Error {
                input: ("*here is italic*"),
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_plaintext("**here is bold**"),
            Err(NomErr::Error(Error {
                input: ("**here is bold**"),
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_plaintext("`here is code`"),
            Err(NomErr::Error(Error {
                input: ("`here is code`"),
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_plaintext("[title](https://www.example.com)"),
            Err(NomErr::Error(Error {
                input: ("[title](https://www.example.com)"),
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_plaintext("![alt text](image.jpg)"),
            Err(NomErr::Error(Error {
                input: ("![alt text](image.jpg)"),
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_plaintext(""),
            Err(NomErr::Error(Error {
                input: (""),
                code: ErrorKind::Not
            }))
        );
    }

    #[test]
    fn test_parse_markdown_inline() {
        assert_eq!(
            parse_markdown_inline("*here is italic*"),
            Ok(((""), MarkdownInline::Italic(String::from("here is italic"))))
        );
        assert_eq!(
            parse_markdown_inline("**here is bold**"),
            Ok(((""), MarkdownInline::Bold(String::from("here is bold"))))
        );
        assert_eq!(
            parse_markdown_inline("`here is code`"),
            Ok((
                (""),
                MarkdownInline::InlineCode(String::from("here is code"))
            ))
        );
        assert_eq!(
            parse_markdown_inline("[title](https://www.example.com)"),
            Ok((
                (""),
                (MarkdownInline::Link(
                    String::from("title"),
                    String::from("https://www.example.com")
                ))
            ))
        );
        assert_eq!(
            parse_markdown_inline("![alt text](image.jpg)"),
            Ok((
                (""),
                (MarkdownInline::Image(String::from("alt text"), String::from("image.jpg")))
            ))
        );
        assert_eq!(
            parse_markdown_inline("here is plaintext!"),
            Ok((
                (""),
                MarkdownInline::Plaintext(String::from("here is plaintext!"))
            ))
        );
        assert_eq!(
            parse_markdown_inline("here is some plaintext *but what if we italicize?"),
            Ok((
                (""),
                MarkdownInline::Plaintext(String::from(
                    "here is some plaintext *but what if we italicize?"
                ))
            ))
        );
        assert_eq!(
            parse_markdown_inline(
                r#"here is some plaintext
    *but what if we italicize?"#
            ),
            Ok((
                ("\n    *but what if we italicize?"),
                MarkdownInline::Plaintext(String::from("here is some plaintext"))
            ))
        );
        assert!(parse_markdown_inline("\n").is_err(),);
        assert!(parse_markdown_inline("").is_err());
    }

    #[test]
    fn test_parse_markdown_text() {
        assert_eq!(parse_markdown_text("\n"), Ok(((""), vec![])));
        assert_eq!(
            parse_markdown_text("here is some plaintext\n"),
            Ok((
                (""),
                vec![MarkdownInline::Plaintext(String::from(
                    "here is some plaintext"
                ))]
            ))
        );
        assert_eq!(
            parse_markdown_text("here is some plaintext\nand some more yeah"),
            Ok((
                ("and some more yeah"),
                vec![MarkdownInline::Plaintext(String::from(
                    "here is some plaintext"
                ))]
            ))
        );
        assert_eq!(
            parse_markdown_text("here is some plaintext *but what if we italicize?*\n"),
            Ok((
                (""),
                vec![
                    MarkdownInline::Plaintext(String::from("here is some plaintext ")),
                    MarkdownInline::Italic(String::from("but what if we italicize?")),
                ]
            ))
        );
        assert_eq!(
                parse_markdown_text("here is some plaintext *but what if we italicize?* I guess it doesnt **matter** in my `code`\n"),
                Ok(((""),vec![
                    MarkdownInline::Plaintext(String::from("here is some plaintext ")),
                    MarkdownInline::Italic(String::from("but what if we italicize?")),
                    MarkdownInline::Plaintext(String::from(" I guess it doesnt ")),
                    MarkdownInline::Bold(String::from("matter")),
                    MarkdownInline::Plaintext(String::from(" in my ")),
                    MarkdownInline::InlineCode(String::from("code")),
                ]))
            );
        assert_eq!(
            parse_markdown_text("here is some plaintext *but what if we italicize?*\n"),
            Ok((
                (""),
                vec![
                    MarkdownInline::Plaintext(String::from("here is some plaintext ")),
                    MarkdownInline::Italic(String::from("but what if we italicize?")),
                ]
            ))
        );
    }

    #[test]
    fn test_parse_header_tag() {
        assert_eq!(parse_header_tag("# "), Ok(((""), 1)));
        assert_eq!(parse_header_tag("### "), Ok(((""), 3)));
        assert_eq!(parse_header_tag("# h1"), Ok((("h1"), 1)));
        assert_eq!(parse_header_tag("# h1"), Ok((("h1"), 1)));
        assert_eq!(
            parse_header_tag(" "),
            Err(NomErr::Error(Error {
                input: (" "),
                code: ErrorKind::TakeWhile1
            }))
        );
        assert_eq!(
            parse_header_tag("#"),
            Err(NomErr::Error(Error {
                input: (""),
                code: ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn test_parse_header() {
        assert_eq!(
            parse_header("# h1\n"),
            Ok((
                (""),
                (1, vec![MarkdownInline::Plaintext(String::from("h1"))])
            ))
        );
        assert_eq!(
            parse_header("## h2\n"),
            Ok((
                (""),
                (2, vec![MarkdownInline::Plaintext(String::from("h2"))])
            ))
        );
        assert_eq!(
            parse_header("###  h3\n"),
            Ok((
                (""),
                (3, vec![MarkdownInline::Plaintext(String::from(" h3"))])
            ))
        );
        assert_eq!(
            parse_header("###h3"),
            Err(NomErr::Error(Error {
                input: ("h3"),
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_header("###"),
            Err(NomErr::Error(Error {
                input: (""),
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_header(""),
            Err(NomErr::Error(Error {
                input: (""),
                code: ErrorKind::TakeWhile1
            }))
        );
        assert_eq!(
            parse_header("#"),
            Err(NomErr::Error(Error {
                input: (""),
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(parse_header("# \n"), Ok(((""), (1, vec![]))));
        assert_eq!(
            parse_header("# test\n"),
            Ok((
                (""),
                (1, vec![MarkdownInline::Plaintext(String::from("test"))])
            ))
        )
    }

    #[test]
    fn test_parse_unordered_list_tag() {
        assert_eq!(parse_unordered_list_tag("- "), Ok(((""), ("-"))));
        assert_eq!(
            parse_unordered_list_tag("- and some more"),
            Ok((("and some more"), ("-")))
        );
        assert_eq!(
            parse_unordered_list_tag("-"),
            Err(NomErr::Error(Error {
                input: (""),
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_unordered_list_tag("-and some more"),
            Err(NomErr::Error(Error {
                input: ("and some more"),
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_unordered_list_tag("--"),
            Err(NomErr::Error(Error {
                input: ("-"),
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_unordered_list_tag(""),
            Err(NomErr::Error(Error {
                input: (""),
                code: ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn test_parse_unordered_list_element() {
        assert_eq!(
            parse_unordered_list_element("- this is an element\n"),
            Ok((
                (""),
                vec![MarkdownInline::Plaintext(String::from(
                    "this is an element"
                ))]
            ))
        );
        assert_eq!(
            parse_unordered_list_element(
                r#"- this is an element
- this is another element
"#
            ),
            Ok((
                ("- this is another element\n"),
                vec![MarkdownInline::Plaintext(String::from(
                    "this is an element"
                ))]
            ))
        );
        assert_eq!(
            parse_unordered_list_element(""),
            Err(NomErr::Error(Error {
                input: (""),
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(parse_unordered_list_element("- \n"), Ok(((""), vec![])));
        assert!(parse_unordered_list_element("- ").is_err());
        assert!(parse_unordered_list_element("- test").is_err());
        assert_eq!(
            parse_unordered_list_element("-"),
            Err(NomErr::Error(Error {
                input: (""),
                code: ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn test_parse_unordered_list() {
        assert!(parse_unordered_list("- this is an element").is_err());
        assert_eq!(
            parse_unordered_list("- this is an element\n"),
            Ok((
                (""),
                vec![vec![MarkdownInline::Plaintext(String::from(
                    "this is an element"
                ))]]
            ))
        );
        assert_eq!(
            parse_unordered_list(
                r#"- this is an element
- here is another
"#
            ),
            Ok((
                (""),
                vec![
                    vec![MarkdownInline::Plaintext(String::from(
                        "this is an element"
                    ))],
                    vec![MarkdownInline::Plaintext(String::from("here is another"))]
                ]
            ))
        );
    }

    #[test]
    fn test_parse_ordered_list_tag() {
        assert_eq!(parse_ordered_list_tag("1. "), Ok(((""), ("1"))));
        assert_eq!(parse_ordered_list_tag("1234567. "), Ok(((""), ("1234567"))));
        assert_eq!(
            parse_ordered_list_tag("3. and some more"),
            Ok((("and some more"), ("3")))
        );
        assert_eq!(
            parse_ordered_list_tag("1"),
            Err(NomErr::Error(Error {
                input: (""),
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_ordered_list_tag("1.and some more"),
            Err(NomErr::Error(Error {
                input: ("and some more"),
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_ordered_list_tag("1111."),
            Err(NomErr::Error(Error {
                input: (""),
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_ordered_list_tag(""),
            Err(NomErr::Error(Error {
                input: (""),
                code: ErrorKind::TakeWhile1
            }))
        );
    }

    #[test]
    fn test_parse_ordered_list_element() {
        assert_eq!(
            parse_ordered_list_element("1. this is an element\n"),
            Ok((
                (""),
                vec![MarkdownInline::Plaintext(String::from(
                    "this is an element"
                ))]
            ))
        );
        assert_eq!(
            parse_ordered_list_element(
                r#"1. this is an element
1. here is another
"#
            ),
            Ok((
                ("1. here is another\n"),
                vec![MarkdownInline::Plaintext(String::from(
                    "this is an element"
                ))]
            ))
        );
        assert_eq!(
            parse_ordered_list_element(""),
            Err(NomErr::Error(Error {
                input: (""),
                code: ErrorKind::TakeWhile1
            }))
        );
        assert_eq!(
            parse_ordered_list_element(""),
            Err(NomErr::Error(Error {
                input: (""),
                code: ErrorKind::TakeWhile1
            }))
        );
        assert_eq!(parse_ordered_list_element("1. \n"), Ok(((""), vec![])));
        assert!(parse_ordered_list_element("1. test").is_err());
        assert!(parse_ordered_list_element("1. ").is_err());
        assert!(parse_ordered_list_element("1.").is_err());
    }

    #[test]
    fn test_parse_ordered_list() {
        assert_eq!(
            parse_ordered_list("1. this is an element\n"),
            Ok((
                (""),
                vec![vec![MarkdownInline::Plaintext(String::from(
                    "this is an element"
                ))]]
            ))
        );
        assert!(parse_ordered_list("1. test").is_err());
        assert_eq!(
            parse_ordered_list(
                r#"1. this is an element
2. here is another
"#
            ),
            Ok((
                (""),
                vec![
                    vec!(MarkdownInline::Plaintext(String::from(
                        "this is an element"
                    ))),
                    vec![MarkdownInline::Plaintext(String::from("here is another"))]
                ]
            ))
        );

        assert_eq!(
            parse_ordered_list(
                r#"1. this is an element
1. here is another
"#
            ),
            Ok((
                (""),
                vec![
                    vec!(MarkdownInline::Plaintext(String::from(
                        "this is an element"
                    ))),
                    vec![MarkdownInline::Plaintext(String::from("here is another"))]
                ]
            ))
        );
    }

    #[test]
    fn test_parse_codeblock() {
        assert_eq!(
            parse_code_block(
                r#"```bash
    pip install foobar
```"#
            ),
            Ok((
                (""),
                (
                    String::from("bash"),
                    String::from(
                        r#"    pip install foobar
"#
                    )
                )
            ))
        );
        assert_eq!(
            parse_code_block(
                r#"```python
    import foobar

    foobar.pluralize('word') # returns 'words'
    foobar.pluralize('goose') # returns 'geese'
    foobar.singularize('phenomena') # returns 'phenomenon'
```"#
            ),
            Ok((
                (""),
                (
                    String::from("python"),
                    String::from(
                        r#"    import foobar

    foobar.pluralize('word') # returns 'words'
    foobar.pluralize('goose') # returns 'geese'
    foobar.singularize('phenomena') # returns 'phenomenon'
"#
                    )
                )
            ))
        );
        assert_eq!(
            parse_code_block(
                r#"```python
    import foobar

    foobar.pluralize('word') # returns 'words'
    foobar.pluralize('goose') # returns 'geese'
    foobar.singularize('phenomena') # returns 'phenomenon'
```
And the rest is here"#
            ),
            Ok((
                ("And the rest is here"),
                (
                    String::from("python"),
                    String::from(
                        r#"    import foobar

    foobar.pluralize('word') # returns 'words'
    foobar.pluralize('goose') # returns 'geese'
    foobar.singularize('phenomena') # returns 'phenomenon'
"#
                    )
                )
            ))
        );
    }

    #[test]
    fn test_parse_markdown() {
        assert_eq!(
            parse_markdown(r#"And that is all folks!"#),
            Ok((
                "",
                vec![Markdown::Line(vec![MarkdownInline::Plaintext(
                    String::from("And that is all folks!")
                )])]
            ))
        );

        assert_eq!(
            parse_markdown(r#"# Digitheque Design Inspiration
## A little smaller

### Third level

#### Fourth level


##### Fifth level, what if this was really long and we were able to cross over lines more than once. Lets try tha tby typig a lot here.
In a hole in the ground there lived a hobbit. Not a nasty, dirty, wet hole, filled with the ends of worms and an oozy smell, nor yet a dry, bare, sandy hole with nothing in it to sit down on or to eat: it was a hobbit-hole, and that means comfort.
###### Lowest Level


### Notes

Colors that could be cool are red `#892B39` and linen `#F5F1E6`

International orange is another option: `#FF4F00`

```sql
My codeblock goes here. why does it 

look weird
```
"#),
            Ok((
                "",
                vec![
                    Markdown::Heading(1, vec![MarkdownInline::Plaintext(String::from("Digitheque Design Inspiration"))]),
                    Markdown::Heading(2, vec![MarkdownInline::Plaintext(String::from("A little smaller"))]),
                    Markdown::Line(vec![]),
                    Markdown::Heading(3, vec![MarkdownInline::Plaintext(String::from("Third level"))]),
                    Markdown::Line(vec![]),
                    Markdown::Heading(4, vec![MarkdownInline::Plaintext(String::from("Fourth level"))]),
                    Markdown::Line(vec![]),
                    Markdown::Line(vec![]),
                    Markdown::Heading(5, vec![MarkdownInline::Plaintext(String::from("Fifth level, what if this was really long and we were able to cross over lines more than once. Lets try tha tby typig a lot here."))]),
                    Markdown::Line(vec![MarkdownInline::Plaintext(String::from("In a hole in the ground there lived a hobbit. Not a nasty, dirty, wet hole, filled with the ends of worms and an oozy smell, nor yet a dry, bare, sandy hole with nothing in it to sit down on or to eat: it was a hobbit-hole, and that means comfort."))]),
                    Markdown::Heading(6, vec![MarkdownInline::Plaintext(String::from("Lowest Level"))]),
                    Markdown::Line(vec![]),
                    Markdown::Line(vec![]),
                    Markdown::Heading(3, vec![MarkdownInline::Plaintext(String::from("Notes"))]),
                    Markdown::Line(vec![]),
                    Markdown::Line(vec![MarkdownInline::Plaintext(String::from("Colors that could be cool are red ")),MarkdownInline::InlineCode(String::from("#892B39")),MarkdownInline::Plaintext(String::from(" and linen ")),MarkdownInline::InlineCode(String::from("#F5F1E6"))]),
                Markdown::Line(vec![]),
                Markdown::Line(vec![MarkdownInline::Plaintext(String::from("International orange is another option: ")),MarkdownInline::InlineCode(String::from("#FF4F00"))]),
                Markdown::Line(vec![]),
                Markdown::Codeblock(String::from("sql"),String::from("My codeblock goes here. why does it \n\nlook weird\n"))
                ]
            ))
        );

        assert_eq!(
            parse_markdown("# Digitheque Design Inspiration\r\n## A little smaller\r\n\r\n### Third level\r\n\r\n#### Fourth level\r\n\r\n\r\n##### Fifth level, what if this was really long and we were able to cross over lines more than once. Lets try tha tby typig a lot here.\r\nIn a hole in the ground there lived a hobbit. Not a nasty, dirty, wet hole, filled with the ends of worms and an oozy smell, nor yet a dry, bare, sandy hole with nothing in it to sit down on or to eat: it was a hobbit-hole, and that means comfort.\r\n###### Lowest Level\r\n\r\n\r\n### Notes\r\n\r\nColors that could be cool are red `#892B39` and linen `#F5F1E6`\r\n\r\nInternational orange is another option: `#FF4F00`\r\n\r\n```sql\r\nMy codeblock goes here. why does it \r\n\r\nlook weird\r\n```\r\n"),
            Ok((
                "",
                vec![
                    Markdown::Heading(1, vec![MarkdownInline::Plaintext(String::from("Digitheque Design Inspiration"))]),
                    Markdown::Heading(2, vec![MarkdownInline::Plaintext(String::from("A little smaller"))]),
                    Markdown::Line(vec![]),
                    Markdown::Heading(3, vec![MarkdownInline::Plaintext(String::from("Third level"))]),
                    Markdown::Line(vec![]),
                    Markdown::Heading(4, vec![MarkdownInline::Plaintext(String::from("Fourth level"))]),
                    Markdown::Line(vec![]),
                    Markdown::Line(vec![]),
                    Markdown::Heading(5, vec![MarkdownInline::Plaintext(String::from("Fifth level, what if this was really long and we were able to cross over lines more than once. Lets try tha tby typig a lot here."))]),
                    Markdown::Line(vec![MarkdownInline::Plaintext(String::from("In a hole in the ground there lived a hobbit. Not a nasty, dirty, wet hole, filled with the ends of worms and an oozy smell, nor yet a dry, bare, sandy hole with nothing in it to sit down on or to eat: it was a hobbit-hole, and that means comfort."))]),
                    Markdown::Heading(6, vec![MarkdownInline::Plaintext(String::from("Lowest Level"))]),
                    Markdown::Line(vec![]),
                    Markdown::Line(vec![]),
                    Markdown::Heading(3, vec![MarkdownInline::Plaintext(String::from("Notes"))]),
                    Markdown::Line(vec![]),
                    Markdown::Line(vec![MarkdownInline::Plaintext(String::from("Colors that could be cool are red ")),MarkdownInline::InlineCode(String::from("#892B39")),MarkdownInline::Plaintext(String::from(" and linen ")),MarkdownInline::InlineCode(String::from("#F5F1E6"))]),
                Markdown::Line(vec![]),
                Markdown::Line(vec![MarkdownInline::Plaintext(String::from("International orange is another option: ")),MarkdownInline::InlineCode(String::from("#FF4F00"))]),
                Markdown::Line(vec![]),
                Markdown::Codeblock(String::from("sql\r"),String::from("My codeblock goes here. why does it \r\n\r\nlook weird\r\n"))
                ]
            ))
        );

        assert_eq!(
            parse_markdown(
                r#"# Foobar

Foobar is a Python library for dealing with word pluralization.

```bash
pip install foobar
```
## Installation

Use the package manager [pip](https://pip.pypa.io/en/stable/) to install foobar.
```python
import foobar

foobar.pluralize('word') # returns 'words'
foobar.pluralize('goose') # returns 'geese'
foobar.singularize('phenomena') # returns 'phenomenon'
```
And that is all folks!"#
            ),
            Ok((
                "",
                vec![
                    Markdown::Heading(1, vec![MarkdownInline::Plaintext(String::from("Foobar"))]),
                    Markdown::Line(vec![]),
                    Markdown::Line(vec![MarkdownInline::Plaintext(String::from(
                        "Foobar is a Python library for dealing with word pluralization."
                    ))]),
                    Markdown::Line(vec![]),
                    Markdown::Codeblock(String::from("bash"), String::from("pip install foobar\n")),
                    Markdown::Heading(
                        2,
                        vec![MarkdownInline::Plaintext(String::from("Installation"))]
                    ),
                    Markdown::Line(vec![]),
                    Markdown::Line(vec![
                        MarkdownInline::Plaintext(String::from("Use the package manager ")),
                        MarkdownInline::Link(
                            String::from("pip"),
                            String::from("https://pip.pypa.io/en/stable/")
                        ),
                        MarkdownInline::Plaintext(String::from(" to install foobar.")),
                    ]),
                    Markdown::Codeblock(
                        String::from("python"),
                        String::from(
                            r#"import foobar

foobar.pluralize('word') # returns 'words'
foobar.pluralize('goose') # returns 'geese'
foobar.singularize('phenomena') # returns 'phenomenon'
"#
                        )
                    ),
                    Markdown::Line(vec![MarkdownInline::Plaintext(String::from(
                        "And that is all folks!"
                    ))])
                ]
            ))
        )
    }
}
