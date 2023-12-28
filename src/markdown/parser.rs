use crate::markdown::Markdown;
use crate::markdown::MarkdownInline;
use crate::markdown::MarkdownText;

use bytes::Bytes;
use nom::combinator::all_consuming;
use nombytes::NomBytes;

use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take, take_while1},
    character::{is_digit, is_newline},
    combinator::{eof, map, not, peek},
    error::{Error, ErrorKind},
    multi::{many0, many1, many_till},
    sequence::{delimited, pair, preceded, terminated, tuple},
    Err as NomErr, IResult,
};

pub fn parse_markdown(i: NomBytes) -> IResult<NomBytes, Vec<Markdown>> {
    many1(alt((
        map(parse_header, |e| Markdown::Heading(e.0, e.1)),
        map(parse_unordered_list, |e| Markdown::UnorderedList(e)),
        map(parse_ordered_list, |e| Markdown::OrderedList(e)),
        map(parse_code_block, |e| Markdown::Codeblock(e.0, e.1)),
        map(parse_markdown_text, |e| Markdown::Line(e)),
        map(parse_markdown_inline, |e| Markdown::Line(vec![e])),
        // map(eof, |e: NomBytes| Markdown::Line(vec![MarkdownInline::Plaintext(e.to_bytes())])),
    )))(i)
}

// **([^*][^*])+**
fn parse_boldtext(i: NomBytes) -> IResult<NomBytes, MarkdownInline> {
    map(
        delimited(tag("**"), is_not("**"), tag("**")),
        |b: NomBytes| MarkdownInline::Bold(b.to_bytes()),
    )(i)
}

// *[^*]+*
fn parse_italics(i: NomBytes) -> IResult<NomBytes, MarkdownInline> {
    map(delimited(tag("*"), is_not("*"), tag("*")), |b: NomBytes| {
        MarkdownInline::Italic(b.to_bytes())
    })(i)
}

// `[^`]+`
fn parse_inline_code(i: NomBytes) -> IResult<NomBytes, MarkdownInline> {
    map(delimited(tag("`"), is_not("`"), tag("`")), |b: NomBytes| {
        MarkdownInline::InlineCode(b.to_bytes())
    })(i)
}

// \[[^\]]+\]\([^\)]\)
fn parse_link(i: NomBytes) -> IResult<NomBytes, MarkdownInline> {
    map(
        pair(
            delimited(tag("["), is_not("]"), tag("]")),
            delimited(tag("("), is_not(")"), tag(")")),
        ),
        |(b, c): (NomBytes, NomBytes)| MarkdownInline::Link(b.to_bytes(), c.to_bytes()),
    )(i)
}

// !\[[^\]]+\]\([^\)]\)
fn parse_image(i: NomBytes) -> IResult<NomBytes, MarkdownInline> {
    map(
        pair(
            delimited(tag("!["), is_not("]"), tag("]")),
            delimited(tag("("), is_not(")"), tag(")")),
        ),
        |(b, c): (NomBytes, NomBytes)| MarkdownInline::Image(b.to_bytes(), c.to_bytes()),
    )(i)
}

// // we want to match many things that are not any of our special tags
// // but since we have no tools available to match and consume in the negative case (without regex)
// // we need to match against our tags, then consume one char
// // we repeat this until we run into one of our special characters
// // then we join our array of characters into a NomBytes
fn parse_plaintext(i: NomBytes) -> IResult<NomBytes, MarkdownInline> {
    let (i, (vec, _)) = many_till(
        take(1u8),
        alt((
            peek(
                alt((
                    parse_boldtext,
                    parse_italics,
                    parse_inline_code,
                    parse_image,
                    parse_link,
                    map(tag("\n"), |t: NomBytes| {
                        MarkdownInline::Plaintext(t.to_bytes())
                    }), 
                    map(eof, |t: NomBytes| {
                        MarkdownInline::Plaintext(t.to_bytes())
                    })           
                ))
            ), 
            
        )),
    )(i)?;

    if vec.len() == 0 {
        Err(NomErr::Error(Error {
            input: i,
            code: ErrorKind::Not,
        }))
    } else {
        Ok((
            i,
            MarkdownInline::Plaintext(Bytes::from(
                vec.into_iter().map(|e| e.to_string()).collect::<String>(),
            )),
        ))
    }
}

fn parse_markdown_inline(i: NomBytes) -> IResult<NomBytes, MarkdownInline> {
    alt((
        parse_italics,
        parse_inline_code,
        parse_boldtext,
        parse_image,
        parse_link,
        parse_plaintext,
    ))(i)
}

fn parse_markdown_text(i: NomBytes) -> IResult<NomBytes, MarkdownText> {
    terminated(many0(parse_markdown_inline), tag("\n"))(i)
}

// #*
fn parse_header_tag(i: NomBytes) -> IResult<NomBytes, usize> {
    map(
        terminated(take_while1(|c| c == b'#'), tag(" ")),
        |s: NomBytes| s.to_bytes().len(),
    )(i)
}

// this combines a tuple of the header tag and the rest of the line
fn parse_header(i: NomBytes) -> IResult<NomBytes, (usize, MarkdownText)> {
    tuple((parse_header_tag, parse_markdown_text))(i)
}

fn parse_unordered_list_tag(i: NomBytes) -> IResult<NomBytes, NomBytes> {
    terminated(tag("-"), tag(" "))(i)
}

fn parse_unordered_list_element(i: NomBytes) -> IResult<NomBytes, MarkdownText> {
    preceded(parse_unordered_list_tag, parse_markdown_text)(i)
}

fn parse_unordered_list(i: NomBytes) -> IResult<NomBytes, Vec<MarkdownText>> {
    many1(parse_unordered_list_element)(i)
}

fn parse_ordered_list_tag(i: NomBytes) -> IResult<NomBytes, NomBytes> {
    terminated(
        terminated(take_while1(|d| is_digit(d as u8)), tag(".")),
        tag(" "),
    )(i)
}

fn parse_ordered_list_element(i: NomBytes) -> IResult<NomBytes, MarkdownText> {
    preceded(parse_ordered_list_tag, parse_markdown_text)(i)
}

fn parse_ordered_list(i: NomBytes) -> IResult<NomBytes, Vec<MarkdownText>> {
    many1(parse_ordered_list_element)(i)
}

fn parse_code_block(i: NomBytes) -> IResult<NomBytes, (Bytes, Bytes)> {
    pair(parse_code_block_lang, parse_code_block_body)(i)
}

fn parse_code_block_body(i: NomBytes) -> IResult<NomBytes, Bytes> {
    map(
        delimited(
            tag("\n"),
            is_not("```"),
            pair(tag("```"), alt((eof, tag("\n")))),
        ),
        |s: NomBytes| s.to_bytes(),
    )(i)
}

fn parse_code_block_lang(i: NomBytes) -> IResult<NomBytes, Bytes> {
    alt((
        preceded(
            tag("```"),
            map(take_while1(|c| !is_newline(c)), |b: NomBytes| b.to_bytes()),
        ),
        map(tag("```"), |_| Bytes::from("__UNKNOWN__")),
    ))(i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::{error::Error, error::ErrorKind, Err as NomErr};

    #[test]
    fn test_parse_italics() {
        assert_eq!(
            parse_italics(NomBytes::from("*here is italic*")),
            Ok((
                NomBytes::from(""),
                MarkdownInline::Italic(Bytes::from("here is italic"))
            ))
        );
        assert_eq!(
            parse_italics(NomBytes::from("*here is italic*\n")),
            Ok((
                NomBytes::from("\n"),
                MarkdownInline::Italic(Bytes::from("here is italic"))
            ))
        );
        assert!(parse_italics(NomBytes::from("*here is italic")).is_err());
        assert!(parse_italics(NomBytes::from("here is italic*")).is_err());
        assert!(parse_italics(NomBytes::from("here is italic")).is_err());
        assert!(parse_italics(NomBytes::from("*")).is_err());
        assert!(parse_italics(NomBytes::from("**")).is_err());
        assert!(parse_italics(NomBytes::from("")).is_err());
        assert!(parse_italics(NomBytes::from("**we are doing bold**")).is_err());
    }

    #[test]
    fn test_parse_boldtext() {
        assert_eq!(
            parse_boldtext(NomBytes::from("**here is bold**")),
            Ok((
                NomBytes::from(""),
                MarkdownInline::Bold(Bytes::from("here is bold"))
            ))
        );
        assert_eq!(
            parse_boldtext(NomBytes::from("**here is bold**\n")),
            Ok((
                NomBytes::from("\n"),
                MarkdownInline::Bold(Bytes::from("here is bold"))
            ))
        );
        assert!(parse_boldtext(NomBytes::from("**here is bold")).is_err());
        assert!(parse_boldtext(NomBytes::from("here is bold**")).is_err());
        assert!(parse_boldtext(NomBytes::from("here is bold")).is_err());
        assert!(parse_boldtext(NomBytes::from("****")).is_err());
        assert!(parse_boldtext(NomBytes::from("**")).is_err());
        assert!(parse_boldtext(NomBytes::from("*")).is_err());
        assert!(parse_boldtext(NomBytes::from("")).is_err());
        assert!(parse_boldtext(NomBytes::from("*this is italic*")).is_err());
    }

    #[test]
    fn test_parse_inline_code() {
        assert_eq!(
            parse_inline_code(NomBytes::from("`here is bold`\n")),
            Ok((
                NomBytes::from("\n"),
                MarkdownInline::InlineCode(Bytes::from("here is bold"))
            ))
        );
        assert!(parse_inline_code(NomBytes::from("`here is code")).is_err());
        assert!(parse_inline_code(NomBytes::from("here is code`")).is_err());
        assert!(parse_inline_code(NomBytes::from("``")).is_err());
        assert!(parse_inline_code(NomBytes::from("`")).is_err());
        assert!(parse_inline_code(NomBytes::from("")).is_err());
    }

    #[test]
    fn test_parse_link() {
        assert_eq!(
            parse_link(NomBytes::from("[title](https://www.example.com)")),
            Ok((
                NomBytes::from(""),
                MarkdownInline::Link(Bytes::from("title"), Bytes::from("https://www.example.com"))
            ))
        );
        assert!(parse_link(NomBytes::from("[title](whatever")).is_err());
    }

    #[test]
    fn test_parse_image() {
        assert_eq!(
            parse_image(NomBytes::from("![alt text](image.jpg)")),
            Ok((
                NomBytes::from(""),
                MarkdownInline::Image(Bytes::from("alt text"), Bytes::from("image.jpg"))
            ))
        );
        assert!(parse_image(NomBytes::from("[title](whatever")).is_err());
    }

    #[test]
    fn test_parse_plaintext() {
        assert_eq!(
            parse_plaintext(NomBytes::from("1234567890")),
            Ok((
                NomBytes::from(""),
                MarkdownInline::Plaintext(Bytes::from("1234567890"))
            ))
        );
        assert_eq!(
            parse_plaintext(NomBytes::from("oh my gosh!")),
            Ok((
                NomBytes::from(""),
                MarkdownInline::Plaintext(Bytes::from("oh my gosh!"))
            ))
        );
        assert_eq!(
            parse_plaintext(NomBytes::from("oh my gosh![")),
            Ok((
                NomBytes::from(""),
                MarkdownInline::Plaintext(Bytes::from("oh my gosh!["))
            ))
        );
        assert_eq!(
            parse_plaintext(NomBytes::from("oh my gosh!*")),
            Ok((
                NomBytes::from(""),
                MarkdownInline::Plaintext(Bytes::from("oh my gosh!*"))
            ))
        );
        assert_eq!(
            parse_plaintext(NomBytes::from("*bold babey bold*")),
            Err(NomErr::Error(Error {
                input: NomBytes::from("*bold babey bold*"),
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_plaintext(NomBytes::from("[link babey](and then somewhat)")),
            Err(NomErr::Error(Error {
                input: NomBytes::from("[link babey](and then somewhat)"),
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_plaintext(NomBytes::from("`codeblock for bums`")),
            Err(NomErr::Error(Error {
                input: NomBytes::from("`codeblock for bums`"),
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_plaintext(NomBytes::from("![ but wait theres more](jk)")),
            Err(NomErr::Error(Error {
                input: NomBytes::from("![ but wait theres more](jk)"),
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_plaintext(NomBytes::from("here is plaintext")),
            Ok((
                NomBytes::from(""),
                MarkdownInline::Plaintext(Bytes::from("here is plaintext"))
            ))
        );
        assert_eq!(
            parse_plaintext(NomBytes::from("here is plaintext!")),
            Ok((
                NomBytes::from(""),
                MarkdownInline::Plaintext(Bytes::from("here is plaintext!"))
            ))
        );
        assert_eq!(
            parse_plaintext(NomBytes::from("here is plaintext![image starting")),
            Ok((
                NomBytes::from(""),
                MarkdownInline::Plaintext(Bytes::from("here is plaintext![image starting"))
            ))
        );
        assert_eq!(
            parse_plaintext(NomBytes::from("here is plaintext\n")),
            Ok((
                NomBytes::from("\n"),
                MarkdownInline::Plaintext(Bytes::from("here is plaintext"))
            ))
        );
        assert_eq!(
            parse_plaintext(NomBytes::from("here is plaintext\nand the next line")),
            Ok((
                NomBytes::from("\nand the next line"),
                MarkdownInline::Plaintext(Bytes::from("here is plaintext"))
            ))
        );
        assert_eq!(
            parse_plaintext(NomBytes::from("*here is italic*")),
            Err(NomErr::Error(Error {
                input: NomBytes::from("*here is italic*"),
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_plaintext(NomBytes::from("**here is bold**")),
            Err(NomErr::Error(Error {
                input: NomBytes::from("**here is bold**"),
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_plaintext(NomBytes::from("`here is code`")),
            Err(NomErr::Error(Error {
                input: NomBytes::from("`here is code`"),
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_plaintext(NomBytes::from("[title](https://www.example.com)")),
            Err(NomErr::Error(Error {
                input: NomBytes::from("[title](https://www.example.com)"),
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_plaintext(NomBytes::from("![alt text](image.jpg)")),
            Err(NomErr::Error(Error {
                input: NomBytes::from("![alt text](image.jpg)"),
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_plaintext(NomBytes::from("")),
            Err(NomErr::Error(Error {
                input: NomBytes::from(""),
                code: ErrorKind::Not
            }))
        );
    }

    #[test]
    fn test_parse_markdown_inline() {
        assert_eq!(
            parse_markdown_inline(NomBytes::from("*here is italic*")),
            Ok((
                NomBytes::from(""),
                MarkdownInline::Italic(Bytes::from("here is italic"))
            ))
        );
        assert_eq!(
            parse_markdown_inline(NomBytes::from("**here is bold**")),
            Ok((
                NomBytes::from(""),
                MarkdownInline::Bold(Bytes::from("here is bold"))
            ))
        );
        assert_eq!(
            parse_markdown_inline(NomBytes::from("`here is code`")),
            Ok((
                NomBytes::from(""),
                MarkdownInline::InlineCode(Bytes::from("here is code"))
            ))
        );
        assert_eq!(
            parse_markdown_inline(NomBytes::from("[title](https://www.example.com)")),
            Ok((
                NomBytes::from(""),
                (MarkdownInline::Link(
                    Bytes::from("title"),
                    Bytes::from("https://www.example.com")
                ))
            ))
        );
        assert_eq!(
            parse_markdown_inline(NomBytes::from("![alt text](image.jpg)")),
            Ok((
                NomBytes::from(""),
                (MarkdownInline::Image(Bytes::from("alt text"), Bytes::from("image.jpg")))
            ))
        );
        assert_eq!(
            parse_markdown_inline(NomBytes::from("here is plaintext!")),
            Ok((
                NomBytes::from(""),
                MarkdownInline::Plaintext(Bytes::from("here is plaintext!"))
            ))
        );
        assert_eq!(
            parse_markdown_inline(NomBytes::from(
                "here is some plaintext *but what if we italicize?"
            )),
            Ok((
                NomBytes::from(""),
                MarkdownInline::Plaintext(Bytes::from(
                    "here is some plaintext *but what if we italicize?"
                ))
            ))
        );
        assert_eq!(
            parse_markdown_inline(NomBytes::from(
                r#"here is some plaintext
    *but what if we italicize?"#
            )),
            Ok((
                NomBytes::from("\n    *but what if we italicize?"),
                MarkdownInline::Plaintext(Bytes::from("here is some plaintext"))
            ))
        );
        assert!(parse_markdown_inline(NomBytes::from("\n")).is_err(),);
        assert!(parse_markdown_inline(NomBytes::from("")).is_err());
    }

    #[test]
    fn test_parse_markdown_text() {
        assert_eq!(
            parse_markdown_text(NomBytes::from("\n")),
            Ok((NomBytes::from(""), vec![]))
        );
        assert_eq!(
            parse_markdown_text(NomBytes::from("here is some plaintext\n")),
            Ok((
                NomBytes::from(""),
                vec![MarkdownInline::Plaintext(Bytes::from(
                    "here is some plaintext"
                ))]
            ))
        );
        assert_eq!(
            parse_markdown_text(NomBytes::from("here is some plaintext\nand some more yeah")),
            Ok((
                NomBytes::from("and some more yeah"),
                vec![MarkdownInline::Plaintext(Bytes::from(
                    "here is some plaintext"
                ))]
            ))
        );
        assert_eq!(
            parse_markdown_text(NomBytes::from(
                "here is some plaintext *but what if we italicize?*\n"
            )),
            Ok((
                NomBytes::from(""),
                vec![
                    MarkdownInline::Plaintext(Bytes::from("here is some plaintext ")),
                    MarkdownInline::Italic(Bytes::from("but what if we italicize?")),
                ]
            ))
        );
        assert_eq!(
                parse_markdown_text(NomBytes::from("here is some plaintext *but what if we italicize?* I guess it doesnt **matter** in my `code`\n")),
                Ok((NomBytes::from(""), vec![
                    MarkdownInline::Plaintext(Bytes::from("here is some plaintext ")),
                    MarkdownInline::Italic(Bytes::from("but what if we italicize?")),
                    MarkdownInline::Plaintext(Bytes::from(" I guess it doesnt ")),
                    MarkdownInline::Bold(Bytes::from("matter")),
                    MarkdownInline::Plaintext(Bytes::from(" in my ")),
                    MarkdownInline::InlineCode(Bytes::from("code")),
                ]))
            );
        assert_eq!(
            parse_markdown_text(NomBytes::from(
                "here is some plaintext *but what if we italicize?*\n"
            )),
            Ok((
                NomBytes::from(""),
                vec![
                    MarkdownInline::Plaintext(Bytes::from("here is some plaintext ")),
                    MarkdownInline::Italic(Bytes::from("but what if we italicize?")),
                ]
            ))
        );
    }

    #[test]
    fn test_parse_header_tag() {
        assert_eq!(
            parse_header_tag(NomBytes::from("# ")),
            Ok((NomBytes::from(""), 1))
        );
        assert_eq!(
            parse_header_tag(NomBytes::from("### ")),
            Ok((NomBytes::from(""), 3))
        );
        assert_eq!(
            parse_header_tag(NomBytes::from("# h1")),
            Ok((NomBytes::from("h1"), 1))
        );
        assert_eq!(
            parse_header_tag(NomBytes::from("# h1")),
            Ok((NomBytes::from("h1"), 1))
        );
        assert_eq!(
            parse_header_tag(NomBytes::from(" ")),
            Err(NomErr::Error(Error {
                input: NomBytes::from(" "),
                code: ErrorKind::TakeWhile1
            }))
        );
        assert_eq!(
            parse_header_tag(NomBytes::from("#")),
            Err(NomErr::Error(Error {
                input: NomBytes::from(""),
                code: ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn test_parse_header() {
        assert_eq!(
            parse_header(NomBytes::from("# h1\n")),
            Ok((
                NomBytes::from(""),
                (1, vec![MarkdownInline::Plaintext(Bytes::from("h1"))])
            ))
        );
        assert_eq!(
            parse_header(NomBytes::from("## h2\n")),
            Ok((
                NomBytes::from(""),
                (2, vec![MarkdownInline::Plaintext(Bytes::from("h2"))])
            ))
        );
        assert_eq!(
            parse_header(NomBytes::from("###  h3\n")),
            Ok((
                NomBytes::from(""),
                (3, vec![MarkdownInline::Plaintext(Bytes::from(" h3"))])
            ))
        );
        assert_eq!(
            parse_header(NomBytes::from("###h3")),
            Err(NomErr::Error(Error {
                input: NomBytes::from("h3"),
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_header(NomBytes::from("###")),
            Err(NomErr::Error(Error {
                input: NomBytes::from(""),
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_header(NomBytes::from("")),
            Err(NomErr::Error(Error {
                input: NomBytes::from(""),
                code: ErrorKind::TakeWhile1
            }))
        );
        assert_eq!(
            parse_header(NomBytes::from("#")),
            Err(NomErr::Error(Error {
                input: NomBytes::from(""),
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_header(NomBytes::from("# \n")),
            Ok((NomBytes::from(""), (1, vec![])))
        );
        assert_eq!(
            parse_header(NomBytes::from("# test\n")),
            Ok((
                NomBytes::from(""),
                (1, vec![MarkdownInline::Plaintext(Bytes::from("test"))])
            ))
        )
    }

    #[test]
    fn test_parse_unordered_list_tag() {
        assert_eq!(
            parse_unordered_list_tag(NomBytes::from("- ")),
            Ok((NomBytes::from(""), NomBytes::from("-")))
        );
        assert_eq!(
            parse_unordered_list_tag(NomBytes::from("- and some more")),
            Ok((NomBytes::from("and some more"), NomBytes::from("-")))
        );
        assert_eq!(
            parse_unordered_list_tag(NomBytes::from("-")),
            Err(NomErr::Error(Error {
                input: NomBytes::from(""),
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_unordered_list_tag(NomBytes::from("-and some more")),
            Err(NomErr::Error(Error {
                input: NomBytes::from("and some more"),
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_unordered_list_tag(NomBytes::from("--")),
            Err(NomErr::Error(Error {
                input: NomBytes::from("-"),
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_unordered_list_tag(NomBytes::from("")),
            Err(NomErr::Error(Error {
                input: NomBytes::from(""),
                code: ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn test_parse_unordered_list_element() {
        assert_eq!(
            parse_unordered_list_element(NomBytes::from("- this is an element\n")),
            Ok((
                NomBytes::from(""),
                vec![MarkdownInline::Plaintext(Bytes::from("this is an element"))]
            ))
        );
        assert_eq!(
            parse_unordered_list_element(NomBytes::from(
                r#"- this is an element
- this is another element
"#
            )),
            Ok((
                NomBytes::from("- this is another element\n"),
                vec![MarkdownInline::Plaintext(Bytes::from("this is an element"))]
            ))
        );
        assert_eq!(
            parse_unordered_list_element(NomBytes::from("")),
            Err(NomErr::Error(Error {
                input: NomBytes::from(""),
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_unordered_list_element(NomBytes::from("- \n")),
            Ok((NomBytes::from(""), vec![]))
        );
        assert!(parse_unordered_list_element(NomBytes::from("- ")).is_err());
        assert!(parse_unordered_list_element(NomBytes::from("- test")).is_err());
        assert_eq!(
            parse_unordered_list_element(NomBytes::from("-")),
            Err(NomErr::Error(Error {
                input: NomBytes::from(""),
                code: ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn test_parse_unordered_list() {
        assert!(parse_unordered_list(NomBytes::from("- this is an element")).is_err());
        assert_eq!(
            parse_unordered_list(NomBytes::from("- this is an element\n")),
            Ok((
                NomBytes::from(""),
                vec![vec![MarkdownInline::Plaintext(Bytes::from(
                    "this is an element"
                ))]]
            ))
        );
        assert_eq!(
            parse_unordered_list(NomBytes::from(
                r#"- this is an element
- here is another
"#
            )),
            Ok((
                NomBytes::from(""),
                vec![
                    vec![MarkdownInline::Plaintext(Bytes::from("this is an element"))],
                    vec![MarkdownInline::Plaintext(Bytes::from("here is another"))]
                ]
            ))
        );
    }

    #[test]
    fn test_parse_ordered_list_tag() {
        assert_eq!(
            parse_ordered_list_tag(NomBytes::from("1. ")),
            Ok((NomBytes::from(""), NomBytes::from("1")))
        );
        assert_eq!(
            parse_ordered_list_tag(NomBytes::from("1234567. ")),
            Ok((NomBytes::from(""), NomBytes::from("1234567")))
        );
        assert_eq!(
            parse_ordered_list_tag(NomBytes::from("3. and some more")),
            Ok((NomBytes::from("and some more"), NomBytes::from("3")))
        );
        assert_eq!(
            parse_ordered_list_tag(NomBytes::from("1")),
            Err(NomErr::Error(Error {
                input: NomBytes::from(""),
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_ordered_list_tag(NomBytes::from("1.and some more")),
            Err(NomErr::Error(Error {
                input: NomBytes::from("and some more"),
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_ordered_list_tag(NomBytes::from("1111.")),
            Err(NomErr::Error(Error {
                input: NomBytes::from(""),
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_ordered_list_tag(NomBytes::from("")),
            Err(NomErr::Error(Error {
                input: NomBytes::from(""),
                code: ErrorKind::TakeWhile1
            }))
        );
    }

    #[test]
    fn test_parse_ordered_list_element() {
        assert_eq!(
            parse_ordered_list_element(NomBytes::from("1. this is an element\n")),
            Ok((
                NomBytes::from(""),
                vec![MarkdownInline::Plaintext(Bytes::from("this is an element"))]
            ))
        );
        assert_eq!(
            parse_ordered_list_element(NomBytes::from(
                r#"1. this is an element
1. here is another
"#
            )),
            Ok((
                NomBytes::from("1. here is another\n"),
                vec![MarkdownInline::Plaintext(Bytes::from("this is an element"))]
            ))
        );
        assert_eq!(
            parse_ordered_list_element(NomBytes::from("")),
            Err(NomErr::Error(Error {
                input: NomBytes::from(""),
                code: ErrorKind::TakeWhile1
            }))
        );
        assert_eq!(
            parse_ordered_list_element(NomBytes::from("")),
            Err(NomErr::Error(Error {
                input: NomBytes::from(""),
                code: ErrorKind::TakeWhile1
            }))
        );
        assert_eq!(
            parse_ordered_list_element(NomBytes::from("1. \n")),
            Ok((NomBytes::from(""), vec![]))
        );
        assert!(parse_ordered_list_element(NomBytes::from("1. test")).is_err());
        assert!(parse_ordered_list_element(NomBytes::from("1. ")).is_err());
        assert!(parse_ordered_list_element(NomBytes::from("1.")).is_err());
    }

    #[test]
    fn test_parse_ordered_list() {
        assert_eq!(
            parse_ordered_list(NomBytes::from("1. this is an element\n")),
            Ok((
                NomBytes::from(""),
                vec![vec![MarkdownInline::Plaintext(Bytes::from(
                    "this is an element"
                ))]]
            ))
        );
        assert!(parse_ordered_list(NomBytes::from("1. test")).is_err());
        assert_eq!(
            parse_ordered_list(NomBytes::from(
                r#"1. this is an element
2. here is another
"#
            )),
            Ok((
                NomBytes::from(""),
                vec![
                    vec!(MarkdownInline::Plaintext(Bytes::from("this is an element"))),
                    vec![MarkdownInline::Plaintext(Bytes::from("here is another"))]
                ]
            ))
        );

        assert_eq!(
            parse_ordered_list(NomBytes::from(
                r#"1. this is an element
1. here is another
"#
            )),
            Ok((
                NomBytes::from(""),
                vec![
                    vec!(MarkdownInline::Plaintext(Bytes::from("this is an element"))),
                    vec![MarkdownInline::Plaintext(Bytes::from("here is another"))]
                ]
            ))
        );
    }

    #[test]
    fn test_parse_codeblock() {
        assert_eq!(
            parse_code_block(NomBytes::from(
                r#"```bash
    pip install foobar
```"#
            )),
            Ok((
                NomBytes::from(""),
                (
                    Bytes::from("bash"),
                    Bytes::from(
                        r#"    pip install foobar
"#
                    )
                )
            ))
        );
        assert_eq!(
            parse_code_block(NomBytes::from(
                r#"```python
    import foobar

    foobar.pluralize('word') # returns 'words'
    foobar.pluralize('goose') # returns 'geese'
    foobar.singularize('phenomena') # returns 'phenomenon'
```"#
            )),
            Ok((
                NomBytes::from(""),
                (
                    Bytes::from("python"),
                    Bytes::from(
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
            parse_code_block(NomBytes::from(
                r#"```python
    import foobar

    foobar.pluralize('word') # returns 'words'
    foobar.pluralize('goose') # returns 'geese'
    foobar.singularize('phenomena') # returns 'phenomenon'
```
And the rest is here"#
            )),
            Ok((
                NomBytes::from("And the rest is here"),
                (
                    Bytes::from("python"),
                    Bytes::from(
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
            parse_markdown(NomBytes::from(r#"And that is all folks!"#)),
            Ok((
                NomBytes::new(Bytes::from("")),
                vec![Markdown::Line(vec![MarkdownInline::Plaintext(
                    Bytes::from("And that is all folks!")
                )])]
            ))
        );

        assert_eq!(
            parse_markdown(NomBytes::from(
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
            )),
            Ok((
                NomBytes::new(Bytes::from("\n")),
                vec![
                    Markdown::Heading(1, vec![MarkdownInline::Plaintext(Bytes::from("Foobar"))]),
                    Markdown::Line(vec![]),
                    Markdown::Line(vec![MarkdownInline::Plaintext(Bytes::from(
                        "Foobar is a Python library for dealing with word pluralization."
                    ))]),
                    Markdown::Line(vec![]),
                    Markdown::Codeblock(Bytes::from("bash"), Bytes::from("pip install foobar\n")),
                    Markdown::Line(vec![]),
                    Markdown::Heading(
                        2,
                        vec![MarkdownInline::Plaintext(Bytes::from("Installation"))]
                    ),
                    Markdown::Line(vec![]),
                    Markdown::Line(vec![
                        MarkdownInline::Plaintext(Bytes::from("Use the package manager ")),
                        MarkdownInline::Link(
                            Bytes::from("pip"),
                            Bytes::from("https://pip.pypa.io/en/stable/")
                        ),
                        MarkdownInline::Plaintext(Bytes::from(" to install foobar.")),
                    ]),
                    Markdown::Codeblock(
                        Bytes::from("python"),
                        Bytes::from(
                            r#"import foobar

foobar.pluralize('word') # returns 'words'
foobar.pluralize('goose') # returns 'geese'
foobar.singularize('phenomena') # returns 'phenomenon'
"#
                        )
                    ),
                    Markdown::Line(vec![MarkdownInline::Plaintext(Bytes::from(
                        "And that is all folks!"
                    ))])
                ]
            ))
        )
    }
}
