pub mod builtin;
pub mod env;
pub mod eval;
pub mod parser;

use env::{Lenv, Lookup};
use std::{error::Error, fmt};

#[derive(Clone)]
pub enum Lval {
    Sym(String),
    Num(f64),
    Sexpr(Vec<Lval>),
    Qexpr(Vec<Lval>),
    Fun(String, Lfun),
    Lambda(Llambda),
    Str(String),
}

impl PartialEq for Lval {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Lval::Sym(a), Lval::Sym(b)) => a == b,
            (Lval::Num(a), Lval::Num(b)) => a == b,
            (Lval::Sexpr(a), Lval::Sexpr(b)) => a == b,
            (Lval::Qexpr(a), Lval::Qexpr(b)) => a == b,
            (Lval::Fun(a, _), Lval::Fun(b, _)) => a == b,
            (Lval::Str(a), Lval::Str(b)) => a == b,
            (Lval::Lambda(a), Lval::Lambda(b)) => a.body == b.body && a.args == b.args,
            _ => false,
        }
    }
}

impl fmt::Display for Lval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Lval::Sym(s) => write!(f, "{}", s),
            Lval::Num(n) => write!(f, "{}", n),
            Lval::Sexpr(s) => write!(
                f,
                "( {} )",
                s.iter()
                    .map(|x| format!("{}", x))
                    .collect::<Vec<String>>()
                    .join(" ")
            ),
            Lval::Qexpr(q) => write!(
                f,
                "[ {} ]",
                q.iter()
                    .map(|x| format!("{}", x))
                    .collect::<Vec<String>>()
                    .join(" ")
            ),
            Lval::Fun(name, _) => write!(f, "{}", name),
            Lval::Str(s) => write!(f, "{}", s),
            Lval::Lambda(l) => write!(
                f,
                "(\\ [{}] [{}])",
                l.args.join(" "),
                l.body
                    .iter()
                    .map(|x| format!("{}", x))
                    .collect::<Vec<String>>()
                    .join(" ")
            ),
        }
    }
}

impl fmt::Debug for Lval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Lval::Sym(s) => write!(f, "{}", s),
            Lval::Num(n) => write!(f, "{}", n),
            Lval::Sexpr(s) => write!(
                f,
                "( {} )",
                s.iter()
                    .map(|x| format!("{}", x))
                    .collect::<Vec<String>>()
                    .join(" ")
            ),
            Lval::Qexpr(q) => write!(
                f,
                "[ {} ]",
                q.iter()
                    .map(|x| format!("{}", x))
                    .collect::<Vec<String>>()
                    .join(" ")
            ),
            Lval::Fun(name, _) => write!(f, "{}", name),
            Lval::Str(s) => write!(f, "{}", s),
            Lval::Lambda(l) => write!(
                f,
                "(\\ [{}] [{}])",
                l.args.join(" "),
                l.body
                    .iter()
                    .map(|x| format!("{}", x))
                    .collect::<Vec<String>>()
                    .join(" ")
            ),
        }
    }
}

#[derive(Clone)]
pub struct Llambda {
    args: Vec<String>,
    body: Vec<Lval>,
    env: Lenv,
}

impl Llambda {
    fn new(args: Vec<String>, body: Vec<Lval>, lookup: Lookup) -> Self {
        let mut lenv = Lenv::new();
        lenv.push(lookup);
        Llambda {
            args,
            body,
            env: lenv,
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct Lerr {
    etype: LerrType,
    details: String,
    message: String,
}

impl Lerr {
    fn new(etype: LerrType, message: String) -> Lerr {
        let msg = match &etype {
            LerrType::DivZero => "Cannot Divide By Zero",
            LerrType::BadOp => "Invalid Operator",
            LerrType::BadNum => "Invalid Operand",
            LerrType::IncorrectParamCount => "Incorrect Number of Params passed to function",
            LerrType::WrongType => "Incorrect Data Type used",
            LerrType::EmptyList => "Empty List passed to function",
            LerrType::UnboundSymbol => "This Symbol has not been Defined",
            LerrType::Interrupt => "User defined Error",
        };

        Lerr {
            details: msg.to_string(),
            message,
            etype,
        }
    }
}

impl fmt::Debug for Lerr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Error: {:?} - {}; {}",
            self.etype, self.details, self.message
        )
    }
}

impl fmt::Display for Lerr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for Lerr {
    fn description(&self) -> &str {
        &self.details
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum LerrType {
    DivZero,
    BadOp,
    BadNum,
    IncorrectParamCount,
    EmptyList,
    WrongType,
    UnboundSymbol,
    Interrupt,
}

pub type Lfun = fn(&mut Lenv, Vec<Lval>) -> Result<Lval, Lerr>;

pub fn add_builtin(env: &mut Lenv, sym: &str, fun: Lfun) {
    env.insert(sym, Lval::Fun(sym.to_string(), fun));
}

fn to_num(expr: Lval) -> Option<f64> {
    if let Lval::Num(n) = expr {
        Some(n)
    } else {
        None
    }
}

fn to_sym(expr: Lval) -> Option<String> {
    if let Lval::Sym(s) = expr {
        Some(s.clone())
    } else {
        None
    }
}

fn to_str(expr: Lval) -> Option<String> {
    if let Lval::Str(s) = expr {
        Some(s.clone())
    } else {
        None
    }
}

fn to_qexpr(expr: Lval) -> Option<Vec<Lval>> {
    if let Lval::Qexpr(s) = expr {
        Some(s.clone())
    } else {
        None
    }
}

#[cfg(test)]
fn to_lambda(expr: &Lval) -> Option<Llambda> {
    if let Lval::Lambda(s) = expr {
        Some(s.clone())
    } else {
        None
    }
}

// pub fn lisp(env: &mut Lenv, input: &str) -> String {
//     // if "env" == input {
//     //     return format!("{:#?}", env.peek().unwrap());
//     // }

//     let ast = parser::root(input);
//     match ast {
//         Ok(tree) => match eval::eval(env, tree.1) {
//             Ok(r) => format!("{:?}", r),
//             Err(r) => format!("{:?}", r),
//         },
//         Err(e) => format!("Error: Parsing Error - Could not parse the input; {}", e),
//     }
// }

pub trait Compile {
    fn from_ast(env: &mut Lenv, ast: Lval) -> Result<String, String>;

    fn from_source(env: &mut Lenv, source: &str) -> Result<String, String> {
        println!("Compiling the source: {}", source);
        let (_, ast) =
            parser::root::<nom::error::VerboseError<&str>>(source).map_err(|e| match e {
                nom::Err::Error(e) | nom::Err::Failure(e) => nom::error::convert_error(source, e),
                _ => String::from("hmm what's this now?"),
            })?;
        println!("{:?}", ast);

        Self::from_ast(env, ast)
    }
}

pub struct Lisp;

impl Compile for Lisp {
    fn from_ast(env: &mut Lenv, ast: Lval) -> Result<String, String> {
        eval::eval(env, ast)
            .map(|v| format!("{:?}", v))
            .map_err(|e| format!("{:?}", e))
    }
}
