use crate::lisp::Lval;
use nom::{
    branch::alt,
    character::complete::{char, multispace0, none_of, one_of},
    combinator::{all_consuming, map},
    error::{context, ContextError, ParseError, ErrorKind},
    multi::{many0, many1},
    number::complete::double,
    sequence::{delimited, preceded},
    IResult,
};

fn parse_number<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    s: &'a str,
) -> IResult<&str, Lval, E> {
    context(
        "Number",
        map(preceded(multispace0, double), |n| Lval::Num(n)),
    )(s)
}

fn parse_symbol<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    s: &'a str,
) -> IResult<&str, Lval, E> {
    context(
        "Symbol",
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
    ))(s)
}

fn parse_string<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    s: &'a str,
) -> IResult<&str, Lval, E> {
    context(
        "String",
        map(
            delimited(
                preceded(multispace0, char('"')),
                many0(map(none_of("\""), |c| format!("{}", c))),
                preceded(multispace0, char('"')),
            ),
            |o| Lval::Str(o.join("")),
        ),
    )(s)
}

fn parse_sexpression<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    s: &'a str,
) -> IResult<&str, Lval, E> {
    context(
        "S-Expression",
        delimited(
            preceded(multispace0, char('(')),
            map(many0(parse_expression), |e| Lval::Sexpr(e)),
            preceded(multispace0, char(')')),
        ),
    )(s)
}

fn parse_qexpression<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    s: &'a str,
) -> IResult<&str, Lval, E> {
    context(
        "Q-Expression",
        delimited(
            preceded(multispace0, char('[')),
            map(many0(parse_expression), |e| Lval::Qexpr(e)),
            preceded(multispace0, char(']')),
        ),
    )(s)
}

fn parse_expression<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    s: &'a str,
) -> IResult<&str, Lval, E> {
    alt((
        parse_number,
        parse_symbol,
        parse_string,
        parse_sexpression,
        parse_qexpression,
    ))(s)
}

pub fn root<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    s: &'a str,
) -> IResult<&str, Lval, E> {
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
        assert_eq!(parse_number::<(&str, ErrorKind)>("1"), Ok(("", Lval::Num(1.0_f64))));
        assert_eq!(
            parse_number::<(&str, ErrorKind)>("1.000001-1"),
            Ok(("-1", Lval::Num(1.000001_f64)))
        );
        assert_eq!(parse_number::<(&str, ErrorKind)>("123E-02"), Ok(("", Lval::Num(1.23_f64))));
        assert_eq!(parse_number::<(&str, ErrorKind)>("-12302"), Ok(("", Lval::Num(-12302_f64))));
        assert_eq!(parse_number::<(&str, ErrorKind)>("  \t1"), Ok(("", Lval::Num(1_f64))));
    }

    #[test]
    fn it_parses_all_symbols() {
        assert_eq!(parse_symbol::<(&str, ErrorKind)>("+"), Ok(("", Lval::Sym(String::from("+")))));
        assert_eq!(parse_symbol::<(&str, ErrorKind)>("\t-"), Ok(("", Lval::Sym(String::from("-")))));
        assert_eq!(parse_symbol::<(&str, ErrorKind)>("  *"), Ok(("", Lval::Sym(String::from("*")))));
        assert_eq!(parse_symbol::<(&str, ErrorKind)>("\n/"), Ok(("", Lval::Sym(String::from("/")))));
        assert_eq!(
            parse_symbol::<(&str, ErrorKind)>("orange"),
            Ok(("", Lval::Sym(String::from("orange"))))
        );
        assert_eq!(
            parse_symbol::<(&str, ErrorKind)>("tail"),
            Ok(("", Lval::Sym(String::from("tail"))))
        );
    }

    #[test]
    fn it_parses_sexpr() {
        assert_eq!(
            parse_sexpression::<(&str, ErrorKind)>(
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
            parse_qexpression::<(&str, ErrorKind)>(
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
            parse_expression::<(&str, ErrorKind)>(
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
            parse_expression::<(&str, ErrorKind)>(
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
            parse_expression::<(&str, ErrorKind)>(
                "9 (* 1
             2 (* 1
          2 3))"
            ),
            Ok((
                " (* 1\n             2 (* 1\n          2 3))",
                Lval::Num(9_f64)
            ))
        );
        assert_eq!(parse_expression::<(&str, ErrorKind)>("1"), Ok(("", Lval::Num(1_f64),)));
        assert_eq!(
            parse_expression::<(&str, ErrorKind)>("*"),
            Ok(("", Lval::Sym(String::from("*"),)))
        );
    }

    #[test]
    fn it_parses_expressions() {
        assert_eq!(
            root::<(&str, ErrorKind)>(
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
        assert_eq!(root::<(&str, ErrorKind)>(""), Ok(("", Lval::Sexpr(vec![]))));
        assert_eq!(
            root::<(&str, ErrorKind)>("()"),
            Ok(("", Lval::Sexpr(vec![Lval::Sexpr(vec![])])))
        );
        assert_eq!(
            root::<(&str, ErrorKind)>("*"),
            Ok(("", Lval::Sexpr(vec![Lval::Sym(String::from("*"))]),))
        );
        assert_eq!(root::<(&str, ErrorKind)>("9"), Ok(("", Lval::Sexpr(vec![Lval::Num(9_f64)]),)));
        assert_eq!(
            root::<(&str, ErrorKind)>("* 1 2 3"),
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
