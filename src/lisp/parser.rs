use crate::lisp::Lval;
use nom::{
    branch::alt,
    character::complete::{char, multispace0, none_of, one_of},
    combinator::{all_consuming, map},
    error::{ErrorKind, ParseError},
    multi::{many0, many1},
    number::complete::double,
    sequence::{delimited, preceded},
    IResult,
};

#[derive(Debug, PartialEq)]
pub enum SyntaxError<I> {
    InvalidArguments,
    InvalidSymbol,
    Nom(I, ErrorKind),
}

impl<I> ParseError<I> for SyntaxError<I> {
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        SyntaxError::Nom(input, kind)
    }

    fn append(_: I, _: ErrorKind, other: Self) -> Self {
        other
    }
}

fn parse_number(s: &str) -> IResult<&str, Lval, SyntaxError<&str>> {
    map(preceded(multispace0, double), |n| Lval::Num(n))(s)
}

fn parse_symbol(s: &str) -> IResult<&str, Lval, SyntaxError<&str>> {
    map(
        preceded(
            multispace0,
            many1(map(
                one_of(
                    "_+\\:-*/=<>|!&%abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890",
                ),
                |c| format!("{}", c),
            )),
        ),
        |o| Lval::Sym(o.join("")),
    )(s)
}

fn parse_string(s: &str) -> IResult<&str, Lval, SyntaxError<&str>> {
    map(
        delimited(
            preceded(multispace0, char('"')),
            many0(map(none_of("\""), |c| format!("{}", c))),
            preceded(multispace0, char('"')),
        ),
        |o| Lval::Str(o.join("")),
    )(s)
}

fn parse_sexpression(s: &str) -> IResult<&str, Lval, SyntaxError<&str>> {
    delimited(
        preceded(multispace0, char('(')),
        map(many0(parse_expression), |e| Lval::Sexpr(e)),
        preceded(multispace0, char(')')),
    )(s)
}

fn parse_qexpression(s: &str) -> IResult<&str, Lval, SyntaxError<&str>> {
    delimited(
        preceded(multispace0, char('[')),
        map(many0(parse_expression), |e| Lval::Qexpr(e)),
        preceded(multispace0, char(']')),
    )(s)
}

fn parse_expression(s: &str) -> IResult<&str, Lval, SyntaxError<&str>> {
    alt((
        parse_number,
        parse_symbol,
        parse_string,
        parse_sexpression,
        parse_qexpression,
    ))(s)
}

pub fn parse(s: &str) -> IResult<&str, Lval, SyntaxError<&str>> {
    all_consuming(delimited(
        multispace0,
        map(many0(parse_expression), |e| Lval::Sexpr(e)),
        multispace0,
    ))(s)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_parses_numbers() {
        assert_eq!(parse_number("1"), Ok(("", Lval::Num(1.0_f64))));
        assert_eq!(
            parse_number("1.000001-1"),
            Ok(("-1", Lval::Num(1.000001_f64)))
        );
        assert_eq!(parse_number("123E-02"), Ok(("", Lval::Num(1.23_f64))));
        assert_eq!(parse_number("-12302"), Ok(("", Lval::Num(-12302_f64))));
        assert_eq!(parse_number("  \t1"), Ok(("", Lval::Num(1_f64))));
    }

    #[test]
    fn it_parses_all_symbols() {
        assert_eq!(parse_symbol("+"), Ok(("", Lval::Sym(String::from("+")))));
        assert_eq!(parse_symbol("\t-"), Ok(("", Lval::Sym(String::from("-")))));
        assert_eq!(parse_symbol("  *"), Ok(("", Lval::Sym(String::from("*")))));
        assert_eq!(parse_symbol("\n/"), Ok(("", Lval::Sym(String::from("/")))));
        assert_eq!(
            parse_symbol("orange"),
            Ok(("", Lval::Sym(String::from("orange"))))
        );
        assert_eq!(
            parse_symbol("tail"),
            Ok(("", Lval::Sym(String::from("tail"))))
        );
    }

    #[test]
    fn it_parses_sexpr() {
        assert_eq!(
            parse_sexpression(
                "(* 1
             2 3)"
            ),
            Ok((
                "",
                Lval::Sexpr(vec!(
                    Lval::Sym(String::from("*")),
                    Lval::Num(1_f64),
                    Lval::Num(2_f64),
                    Lval::Num(3_f64),
                ))
            ))
        );
    }

    #[test]
    fn it_parses_qexpr() {
        assert_eq!(
            parse_qexpression(
                "[* 1
             2 3]"
            ),
            Ok((
                "",
                Lval::Qexpr(vec!(
                    Lval::Sym(String::from("*")),
                    Lval::Num(1_f64),
                    Lval::Num(2_f64),
                    Lval::Num(3_f64),
                ))
            ))
        );
    }

    #[test]
    fn it_parses_an_expression() {
        assert_eq!(
            parse_expression(
                "(* 1
             2 3)"
            ),
            Ok((
                "",
                Lval::Sexpr(vec!(
                    Lval::Sym(String::from("*")),
                    Lval::Num(1_f64),
                    Lval::Num(2_f64),
                    Lval::Num(3_f64),
                ))
            ))
        );

        assert_eq!(
            parse_expression(
                "(* 1
             2 (* 1
          2 3))"
            ),
            Ok((
                "",
                Lval::Sexpr(vec!(
                    Lval::Sym(String::from("*")),
                    Lval::Num(1_f64),
                    Lval::Num(2_f64),
                    Lval::Sexpr(vec!(
                        Lval::Sym(String::from("*")),
                        Lval::Num(1_f64),
                        Lval::Num(2_f64),
                        Lval::Num(3_f64),
                    )),
                ))
            ))
        );

        assert_eq!(
            parse_expression(
                "9 (* 1
             2 (* 1
          2 3))"
            ),
            Ok((
                " (* 1\n             2 (* 1\n          2 3))",
                Lval::Num(9_f64)
            ))
        );
        assert_eq!(parse_expression("1"), Ok(("", Lval::Num(1_f64),)));
        assert_eq!(
            parse_expression("*"),
            Ok(("", Lval::Sym(String::from("*"),)))
        );
    }

    #[test]
    fn it_parses_expressions() {
        assert_eq!(
            parse(
                "* 9 (* 1
             2 (* 1
          2 3))"
            ),
            Ok((
                "",
                Lval::Sexpr(vec!(
                    Lval::Sym(String::from("*")),
                    Lval::Num(9_f64),
                    Lval::Sexpr(vec!(
                        Lval::Sym(String::from("*")),
                        Lval::Num(1_f64),
                        Lval::Num(2_f64),
                        Lval::Sexpr(vec!(
                            Lval::Sym(String::from("*")),
                            Lval::Num(1_f64),
                            Lval::Num(2_f64),
                            Lval::Num(3_f64),
                        )),
                    )),
                ))
            ))
        );
        assert_eq!(parse(""), Ok(("", Lval::Sexpr(vec![]))));
        assert_eq!(
            parse("()"),
            Ok(("", Lval::Sexpr(vec![Lval::Sexpr(vec![])])))
        );
        assert_eq!(
            parse("*"),
            Ok(("", Lval::Sexpr(vec![Lval::Sym(String::from("*"))]),))
        );
        assert_eq!(parse("9"), Ok(("", Lval::Sexpr(vec![Lval::Num(9_f64)]),)));
        assert_eq!(
            parse("* 1 2 3"),
            Ok((
                "",
                Lval::Sexpr(vec!(
                    Lval::Sym(String::from("*")),
                    Lval::Num(1_f64),
                    Lval::Num(2_f64),
                    Lval::Num(3_f64),
                )),
            ))
        );
    }
}
