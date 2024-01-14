use std::time::{SystemTime, UNIX_EPOCH, Duration};
use crate::lisp::{
    add_builtin, eval, to_num, to_qexpr, to_str, to_sym, Lenv, Lerr, LerrType, Llambda, Lval,
};

pub fn init_builtins(env: &mut Lenv) {
    add_builtin(env, "!", builtin_not);
    add_builtin(env, "+", builtin_add);
    add_builtin(env, "-", builtin_sub);
    add_builtin(env, "*", builtin_mul);
    add_builtin(env, "/", builtin_div);
    add_builtin(env, "%", builtin_mod);

    add_builtin(env, "head", builtin_head);
    add_builtin(env, "tail", builtin_tail);
    add_builtin(env, "list", builtin_list);
    add_builtin(env, "eval", builtin_eval);
    add_builtin(env, "join", builtin_join);
    add_builtin(env, "concat", builtin_concat);

    add_builtin(env, "\\", builtin_lambda);
    add_builtin(env, "def", builtin_def);
    add_builtin(env, "=", builtin_var);

    add_builtin(env, "if", builtin_if);
    add_builtin(env, "echo", builtin_echo);
    add_builtin(env, "rand", builtin_rand);

    add_builtin(env, "die", builtin_err);

    add_builtin(env, "<", builtin_lt);
    add_builtin(env, ">", builtin_gt);
    add_builtin(env, ">=", builtin_gte);
    add_builtin(env, "<=", builtin_lte);
    add_builtin(env, "==", builtin_eq);
    add_builtin(env, "!=", builtin_ne);
    add_builtin(env, "&&", builtin_and);
    add_builtin(env, "||", builtin_or);
}

fn builtin_op(sym: &str, operands: Vec<Lval>) -> Result<Lval, Lerr> {
    // cast everything into a number
    let numbers = operands
        .into_iter()
        .map(to_num)
        .collect::<Option<Vec<f64>>>()
        .ok_or(Lerr::new(
            LerrType::BadNum,
            format!("Function {} can operate only on numbers", sym),
        ))?;

    // handle unary functions
    if numbers.len() == 1 {
        if "-" == sym {
            return Ok(Lval::Num(-numbers[0]));
        } else if "!" == sym {
            let n = if numbers[0] == 0_f64 { 1_f64 } else { 0_f64 };
            return Ok(Lval::Num(n));
        } else {
            return Ok(Lval::Num(numbers[0]));
        }
    }

    let mut x = numbers[0];
    let mut i = 1;

    // apply the symbol over each operand
    while i < numbers.len() {
        let y = numbers[i];
        match sym {
            "-" => x -= y,
            "*" => x *= y,
            "%" => x %= y,
            "/" => {
                if y == 0_f64 {
                    return Err(Lerr::new(
                        LerrType::DivZero,
                        format!("You cannot divide {}, or any number, by 0", x),
                    ));
                } else {
                    x /= y;
                }
            }
            _ => x += y,
        }
        i += 1;
    }

    Ok(Lval::Num(x))
}

fn builtin_ord(sym: &str, operands: Vec<Lval>) -> Result<Lval, Lerr> {
    // need exactly two operands
    if operands.len() != 2 {
        return Err(Lerr::new(
            LerrType::IncorrectParamCount,
            format!(
                "Function {} needed 2 args but was given {}",
                sym,
                operands.len()
            ),
        ));
    }

    // cast everything into a number
    let numbers = operands
        .into_iter()
        .map(to_num)
        .collect::<Option<Vec<f64>>>()
        .ok_or(Lerr::new(
            LerrType::BadNum,
            format!("Function {} can operate only on numbers", sym),
        ))?;

    let x = numbers[0];
    let y = numbers[1];

    // these are for booleans
    let a = if x == 0_f64 { false } else { true };
    let b = if y == 0_f64 { false } else { true };

    let r = match sym {
        ">" => x > y,
        "<" => x < y,
        ">=" => x >= y,
        "<=" => x <= y,
        "&&" => a && b,
        "||" => a || b,
        _ => false,
    };

    if r {
        Ok(Lval::Num(1_f64))
    } else {
        Ok(Lval::Num(0_f64))
    }
}

fn builtin_eq(_env: &mut Lenv, operands: Vec<Lval>) -> Result<Lval, Lerr> {
    // need exactly two operands
    if operands.len() != 2 {
        return Err(Lerr::new(
            LerrType::IncorrectParamCount,
            format!("Function eq needed 2 arg but was given {}", operands.len()),
        ));
    }

    if operands[0] == operands[1] {
        Ok(Lval::Num(1_f64))
    } else {
        Ok(Lval::Num(0_f64))
    }
}

fn builtin_ne(_env: &mut Lenv, operands: Vec<Lval>) -> Result<Lval, Lerr> {
    // need exactly two operands
    if operands.len() != 2 {
        return Err(Lerr::new(
            LerrType::IncorrectParamCount,
            format!("Function eq needed 2 arg but was given {}", operands.len()),
        ));
    }

    if operands[0] == operands[1] {
        Ok(Lval::Num(0_f64))
    } else {
        Ok(Lval::Num(1_f64))
    }
}

fn builtin_gt(_env: &mut Lenv, operands: Vec<Lval>) -> Result<Lval, Lerr> {
    builtin_ord(">", operands)
}

fn builtin_lt(_env: &mut Lenv, operands: Vec<Lval>) -> Result<Lval, Lerr> {
    builtin_ord("<", operands)
}

fn builtin_gte(_env: &mut Lenv, operands: Vec<Lval>) -> Result<Lval, Lerr> {
    builtin_ord(">=", operands)
}

fn builtin_lte(_env: &mut Lenv, operands: Vec<Lval>) -> Result<Lval, Lerr> {
    builtin_ord("<=", operands)
}

fn builtin_and(_env: &mut Lenv, operands: Vec<Lval>) -> Result<Lval, Lerr> {
    builtin_ord("&&", operands)
}

fn builtin_or(_env: &mut Lenv, operands: Vec<Lval>) -> Result<Lval, Lerr> {
    builtin_ord("||", operands)
}

fn builtin_not(_env: &mut Lenv, operands: Vec<Lval>) -> Result<Lval, Lerr> {
    builtin_op("!", operands)
}

fn builtin_add(_env: &mut Lenv, operands: Vec<Lval>) -> Result<Lval, Lerr> {
    builtin_op("+", operands)
}

fn builtin_sub(_env: &mut Lenv, operands: Vec<Lval>) -> Result<Lval, Lerr> {
    builtin_op("-", operands)
}

fn builtin_mul(_env: &mut Lenv, operands: Vec<Lval>) -> Result<Lval, Lerr> {
    builtin_op("*", operands)
}

fn builtin_mod(_env: &mut Lenv, operands: Vec<Lval>) -> Result<Lval, Lerr> {
    builtin_op("%", operands)
}

fn builtin_div(_env: &mut Lenv, operands: Vec<Lval>) -> Result<Lval, Lerr> {
    builtin_op("/", operands)
}

fn builtin_rand(_env: &mut Lenv, operands: Vec<Lval>) -> Result<Lval, Lerr> {
    if operands.len() != 0 {
        return Err(Lerr::new(
            LerrType::IncorrectParamCount,
            format!("Function if needed 0 arg but was given {}", operands.len()),
        ));
    }

    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or(Duration::from_nanos(12345)).subsec_nanos();
    Ok(Lval::Num(nanos as f64))
}

fn builtin_if(env: &mut Lenv, operands: Vec<Lval>) -> Result<Lval, Lerr> {
    if operands.len() != 3 {
        return Err(Lerr::new(
            LerrType::IncorrectParamCount,
            format!("Function if needed 3 arg but was given {}", operands.len()),
        ));
    }

    let conditional = to_num(operands[0].clone()).ok_or(Lerr::new(
        LerrType::WrongType,
        format!(
            "Function if needed conditional but was given {:?}",
            operands[0]
        ),
    ))?;

    let then = to_qexpr(operands[1].clone()).ok_or(Lerr::new(
        LerrType::WrongType,
        format!(
            "Function if needed qexpr for Then but was given {:?}",
            operands[1]
        ),
    ))?;

    let els = to_qexpr(operands[2].clone()).ok_or(Lerr::new(
        LerrType::WrongType,
        format!(
            "Function if needed qexpr for Else but was given {:?}",
            operands[2]
        ),
    ))?;

    if conditional == 0_f64 {
        eval::eval(env, Lval::Sexpr(els))
    } else {
        eval::eval(env, Lval::Sexpr(then))
    }
}

fn builtin_err(_env: &mut Lenv, operands: Vec<Lval>) -> Result<Lval, Lerr> {
    let err = to_str(operands[0].clone()).ok_or(Lerr::new(
        LerrType::WrongType,
        format!(
            "Function die needed qexpr for Else but was given {:?}",
            operands[0]
        ),
    ))?;

    Err(Lerr::new(LerrType::Interrupt, err))
}

fn builtin_head(_env: &mut Lenv, operands: Vec<Lval>) -> Result<Lval, Lerr> {
    // we want only one arguement
    if operands.len() != 1 {
        return Err(Lerr::new(
            LerrType::IncorrectParamCount,
            format!(
                "Function head needed 1 arg but was given {}",
                operands.len()
            ),
        ));
    }

    let arg = &operands[0];
    // need a list/qexpr to work with
    match arg {
        Lval::Qexpr(qexpr) => {
            if qexpr.len() == 0 {
                Err(Lerr::new(
                    LerrType::EmptyList,
                    format!("Function head was given empty list"),
                ))
            } else {
                Ok(qexpr[0].clone())
            }
        }
        _ => Err(Lerr::new(
            LerrType::WrongType,
            format!("Function head needed Qexpr but was given {:?}", arg),
        )),
    }
}

fn builtin_tail(_env: &mut Lenv, operands: Vec<Lval>) -> Result<Lval, Lerr> {
    // we want only one arguement
    if operands.len() != 1 {
        return Err(Lerr::new(
            LerrType::IncorrectParamCount,
            format!(
                "Function tail needed 1 arg but was given {}",
                operands.len()
            ),
        ));
    }

    let arg = &operands[0];
    // need a list/qexpr to work with
    match arg {
        Lval::Qexpr(qexpr) => {
            if qexpr.len() == 0 {
                Err(Lerr::new(
                    LerrType::EmptyList,
                    format!("Function tail was given empty list"),
                ))
            } else {
                Ok(Lval::Qexpr(qexpr[1..].to_vec()))
            }
        }
        _ => Err(Lerr::new(
            LerrType::WrongType,
            format!("Function tail needed Qexpr but was given {:?}", arg),
        )),
    }
}

fn builtin_list(_env: &mut Lenv, operands: Vec<Lval>) -> Result<Lval, Lerr> {
    Ok(Lval::Qexpr(operands))
}

fn builtin_eval(env: &mut Lenv, operands: Vec<Lval>) -> Result<Lval, Lerr> {
    // we only want to evaluate one arguement
    if operands.len() != 1 {
        return Err(Lerr::new(
            LerrType::IncorrectParamCount,
            format!(
                "Function eval needed 1 arg but was given {}",
                operands.len()
            ),
        ));
    }

    let arg = &operands[0];
    match arg {
        Lval::Qexpr(qexpr) => eval::eval(env, Lval::Sexpr(qexpr[..].to_vec())),
        _ => eval::eval(env, arg.clone()),
    }
}

fn builtin_echo(env: &mut Lenv, operands: Vec<Lval>) -> Result<Lval, Lerr> {
    // we only want to evaluate one arguement
    if operands.len() != 1 {
        return Err(Lerr::new(
            LerrType::IncorrectParamCount,
            format!(
                "Function echo needed 1 arg but was given {}",
                operands.len()
            ),
        ));
    }

    let arg = &operands[0];
    Ok(Lval::Str(format!("\"{:?}\"", arg)))
}

fn builtin_join(_env: &mut Lenv, operands: Vec<Lval>) -> Result<Lval, Lerr> {
    // need at least 2 arguements
    if operands.len() < 2 {
        return Err(Lerr::new(
            LerrType::IncorrectParamCount,
            format!(
                "Function join needed 2 arg but was given {}",
                operands.len()
            ),
        ));
    }

    // cast everything into a qexppr
    let qexprs = operands
        .into_iter()
        .map(to_qexpr)
        .collect::<Option<Vec<_>>>()
        .ok_or(Lerr::new(
            LerrType::WrongType,
            format!("Function join needed Qexpr but was given"),
        ))?;

    // push each elements from each arguements into one qexpr
    let mut joined = vec![];
    for qexp in qexprs {
        for item in qexp {
            joined.push(item);
        }
    }

    Ok(Lval::Qexpr(joined))
}

fn builtin_concat(_env: &mut Lenv, operands: Vec<Lval>) -> Result<Lval, Lerr> {
    // need at least 1 arguements
    if operands.len() < 1 {
        return Err(Lerr::new(
            LerrType::IncorrectParamCount,
            format!(
                "Function concat needed >= 1 arg but was given {}",
                operands.len()
            ),
        ));
    }

    // cast everything into a qexppr
    let strings = operands
        .into_iter()
        .map(to_str)
        .collect::<Option<Vec<_>>>()
        .ok_or(Lerr::new(
            LerrType::WrongType,
            format!("Function concat needed Strings but was given"),
        ))?;

    // push each elements from each arguements into one string
    let mut concatted = String::from("");
    for string in strings {
        concatted = format!("{}{}", concatted, string);
    }

    Ok(Lval::Str(concatted))
}

fn builtin_def(env: &mut Lenv, operands: Vec<Lval>) -> Result<Lval, Lerr> {
    builtin_assign("def", env, operands)
}

fn builtin_var(env: &mut Lenv, operands: Vec<Lval>) -> Result<Lval, Lerr> {
    builtin_assign("=", env, operands)
}

fn builtin_assign(sym: &str, env: &mut Lenv, operands: Vec<Lval>) -> Result<Lval, Lerr> {
    // need at least an arguement list and a value
    if operands.len() < 2 {
        return Err(Lerr::new(
            LerrType::IncorrectParamCount,
            format!(
                "Function def needed 2 args but was given {}",
                operands.len()
            ),
        ));
    }

    let args = operands[0].clone();

    // need each argument to be a symbol
    let args = to_qexpr(args)
        .ok_or(Lerr::new(
            LerrType::WrongType,
            format!("Function def needed Qexpr but was given {:?}", operands[0]),
        ))?
        .into_iter()
        .map(to_sym)
        .collect::<Option<Vec<String>>>()
        .ok_or(Lerr::new(
            LerrType::WrongType,
            format!("Function def needed a param list of all Symbols"),
        ))?;

    // need to have the same number of args and values to assign
    if args.len() != operands.len() - 1 {
        return Err(Lerr::new(
            LerrType::IncorrectParamCount,
            format!(
                "Function def needed to assign {} values but was passed {}",
                args.len(),
                operands.len() - 1
            ),
        ));
    }

    // assign each arg to a corresponding value
    for (i, arg) in args.into_iter().enumerate() {
        if sym == "def" {
            env.insert_last(&arg, operands[i + 1].clone());
        } else {
            env.insert(&arg, operands[i + 1].clone());
        }
    }

    Ok(Lval::Str(String::from("")))
}

fn builtin_lambda(env: &mut Lenv, operands: Vec<Lval>) -> Result<Lval, Lerr> {
    if operands.len() != 2 {
        return Err(Lerr::new(
            LerrType::IncorrectParamCount,
            format!("Function \\ needed 2 arg but was given {}", operands.len()),
        ));
    }

    // needs all arguements to be qexpr
    let results = operands
        .into_iter()
        .map(to_qexpr)
        .collect::<Option<Vec<_>>>()
        .ok_or(Lerr::new(
            LerrType::WrongType,
            format!("Function \\ needed a Qexpr for arguments and a Qexpr for body"),
        ))?;

    let args = results[0].clone();
    // need each argument to be a symbol
    let args = args
        .into_iter()
        .map(to_sym)
        .collect::<Option<Vec<String>>>()
        .ok_or(Lerr::new(
            LerrType::WrongType,
            format!("Function \\ needed a param list of all Symbols"),
        ))?;

    let body = results[1].clone();
    let new_env = env.peek().unwrap().clone();
    let lambda = Llambda::new(args, body, new_env);

    Ok(Lval::Lambda(lambda))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lisp::{env::init_env, to_lambda};

    fn empty_fun(_env: &mut Lenv, _operands: Vec<Lval>) -> Result<Lval, Lerr> {
        Ok(Lval::Sexpr(vec![]))
    }

    #[test]
    fn it_correctly_uses_head() {
        let env = &mut init_env();
        let expr = Lval::Qexpr(vec![
            Lval::Sym(String::from("+")),
            Lval::Num(1_f64),
            Lval::Sexpr(vec![
                Lval::Sym(String::from("+")),
                Lval::Num(1_f64),
                Lval::Num(1_f64),
            ]),
        ]);
        assert_eq!(
            builtin_head(env, vec![expr.clone()]).unwrap(),
            Lval::Sym(String::from("+"))
        );

        let _ = builtin_head(env, vec![])
            .map_err(|err| assert_eq!(err.etype, LerrType::IncorrectParamCount));

        let _ = builtin_head(env, vec![Lval::Sym(String::from("+"))])
            .map_err(|err| assert_eq!(err.etype, LerrType::WrongType));

        let _ = builtin_head(env, vec![Lval::Qexpr(vec![])])
            .map_err(|err| assert_eq!(err.etype, LerrType::EmptyList));
    }

    #[test]
    fn it_correctly_uses_tail() {
        let env = &mut init_env();
        let expr = Lval::Qexpr(vec![
            Lval::Sym(String::from("+")),
            Lval::Num(1_f64),
            Lval::Sexpr(vec![
                Lval::Sym(String::from("+")),
                Lval::Num(1_f64),
                Lval::Num(1_f64),
            ]),
        ]);
        assert_eq!(
            builtin_tail(env, vec![expr.clone()]).unwrap(),
            Lval::Qexpr(vec![
                Lval::Num(1_f64),
                Lval::Sexpr(vec![
                    Lval::Sym(String::from("+")),
                    Lval::Num(1_f64),
                    Lval::Num(1_f64),
                ])
            ])
        );
        let _ = builtin_tail(env, vec![])
            .map_err(|err| assert_eq!(err.etype, LerrType::IncorrectParamCount));

        let _ = builtin_tail(env, vec![Lval::Sym(String::from("+"))])
            .map_err(|err| assert_eq!(err.etype, LerrType::WrongType));

        let _ = builtin_tail(env, vec![Lval::Qexpr(vec![])])
            .map_err(|err| assert_eq!(err.etype, LerrType::EmptyList));
    }

    #[test]
    fn it_correctly_uses_list() {
        let env = &mut init_env();
        let expr = vec![
            Lval::Sym(String::from("+")),
            Lval::Num(1_f64),
            Lval::Sexpr(vec![
                Lval::Sym(String::from("+")),
                Lval::Num(1_f64),
                Lval::Num(1_f64),
            ]),
        ];
        assert_eq!(
            builtin_list(env, expr.clone()).unwrap(),
            Lval::Qexpr(vec![
                Lval::Sym(String::from("+")),
                Lval::Num(1_f64),
                Lval::Sexpr(vec![
                    Lval::Sym(String::from("+")),
                    Lval::Num(1_f64),
                    Lval::Num(1_f64),
                ])
            ])
        );
        assert_eq!(
            builtin_list(
                env,
                vec![
                    Lval::Sym(String::from("+")),
                    Lval::Num(1_f64),
                    Lval::Num(1_f64),
                ]
            )
            .unwrap(),
            Lval::Qexpr(vec![
                Lval::Sym(String::from("+")),
                Lval::Num(1_f64),
                Lval::Num(1_f64),
            ])
        );
        assert_eq!(builtin_list(env, vec![]).unwrap(), Lval::Qexpr(vec![]));
        assert_eq!(
            builtin_list(env, vec![Lval::Sym(String::from("+"))]).unwrap(),
            Lval::Qexpr(vec![Lval::Sym(String::from("+")),])
        );
        assert_eq!(
            builtin_list(env, vec![Lval::Sexpr(vec![])]).unwrap(),
            Lval::Qexpr(vec![Lval::Sexpr(vec![]),])
        );
    }

    #[test]
    fn it_correctly_uses_eval() {
        let env = &mut init_env();
        let expr = Lval::Qexpr(vec![
            Lval::Sym(String::from("+")),
            Lval::Num(1_f64),
            Lval::Sexpr(vec![
                Lval::Sym(String::from("+")),
                Lval::Num(1_f64),
                Lval::Num(1_f64),
            ]),
        ]);
        assert_eq!(
            builtin_eval(env, vec![expr.clone()]).unwrap(),
            Lval::Num(3_f64)
        );

        let _ = builtin_eval(env, vec![expr.clone(), expr.clone()])
            .map_err(|err| assert_eq!(err.etype, LerrType::IncorrectParamCount));

        let _ = builtin_eval(env, vec![])
            .map_err(|err| assert_eq!(err.etype, LerrType::IncorrectParamCount));

        assert_eq!(
            builtin_eval(env, vec![Lval::Sym(String::from("-"))]).unwrap(),
            Lval::Fun(empty_fun)
        );
        assert_eq!(
            builtin_eval(env, vec![Lval::Sexpr(vec![Lval::Sym(String::from("-"))])]).unwrap(),
            Lval::Fun(empty_fun)
        );
        assert_eq!(
            builtin_eval(env, vec![Lval::Qexpr(vec![])]).unwrap(),
            Lval::Sexpr(vec![])
        );
    }

    #[test]
    fn it_correctly_uses_join() {
        let env = &mut init_env();
        let expr = Lval::Qexpr(vec![
            Lval::Sym(String::from("+")),
            Lval::Num(1_f64),
            Lval::Sexpr(vec![
                Lval::Sym(String::from("+")),
                Lval::Num(1_f64),
                Lval::Num(1_f64),
            ]),
        ]);
        assert_eq!(
            builtin_join(env, vec![expr.clone(), expr.clone()]).unwrap(),
            Lval::Qexpr(vec![
                Lval::Sym(String::from("+")),
                Lval::Num(1_f64),
                Lval::Sexpr(vec![
                    Lval::Sym(String::from("+")),
                    Lval::Num(1_f64),
                    Lval::Num(1_f64),
                ]),
                Lval::Sym(String::from("+")),
                Lval::Num(1_f64),
                Lval::Sexpr(vec![
                    Lval::Sym(String::from("+")),
                    Lval::Num(1_f64),
                    Lval::Num(1_f64),
                ]),
            ])
        );

        let _ = builtin_join(env, vec![expr.clone()])
            .map_err(|err| assert_eq!(err.etype, LerrType::IncorrectParamCount));

        let _ = builtin_join(env, vec![])
            .map_err(|err| assert_eq!(err.etype, LerrType::IncorrectParamCount));

        let _ = builtin_join(env, vec![expr.clone(), Lval::Sym(String::from("+"))])
            .map_err(|err| assert_eq!(err.etype, LerrType::WrongType));

        assert_eq!(
            builtin_join(env, vec![expr.clone(), Lval::Qexpr(vec![])]).unwrap(),
            Lval::Qexpr(vec![
                Lval::Sym(String::from("+")),
                Lval::Num(1_f64),
                Lval::Sexpr(vec![
                    Lval::Sym(String::from("+")),
                    Lval::Num(1_f64),
                    Lval::Num(1_f64),
                ]),
            ])
        );
    }

    #[test]
    fn it_correctly_uses_concat() {
        let env = &mut init_env();
        assert_eq!(
            builtin_concat(
                env,
                vec![
                    Lval::Str(String::from("ceci")),
                    Lval::Str(String::from(" n'est")),
                    Lval::Str(String::from(" pas")),
                    Lval::Str(String::from(" une")),
                    Lval::Str(String::from(" pipe"))
                ]
            )
            .unwrap(),
            Lval::Str(String::from("ceci n'est pas une pipe"))
        );
    }

    #[test]
    fn it_correctly_uses_define() {
        let env = &mut init_env();
        assert_eq!(
            builtin_def(
                env,
                vec![
                    Lval::Qexpr(vec![
                        Lval::Sym(String::from("a")),
                        Lval::Sym(String::from("b")),
                        Lval::Sym(String::from("c"))
                    ]),
                    Lval::Num(1_f64),
                    Lval::Sym(String::from("+")),
                    Lval::Sexpr(vec![]),
                ]
            )
            .unwrap(),
            Lval::Sexpr(vec![])
        );
        assert_eq!(
            crate::lisp::eval::eval(env, Lval::Sym(String::from("a"))).unwrap(),
            Lval::Num(1_f64)
        );
        assert_eq!(
            crate::lisp::eval::eval(env, Lval::Sym(String::from("b"))).unwrap(),
            Lval::Sym(String::from("+"))
        );
        assert_eq!(
            crate::lisp::eval::eval(env, Lval::Sym(String::from("c"))).unwrap(),
            Lval::Sexpr(vec![])
        );
        let _ = builtin_def(
            env,
            vec![Lval::Qexpr(vec![
                Lval::Sym(String::from("a")),
                Lval::Sym(String::from("b")),
                Lval::Sym(String::from("c")),
            ])],
        )
        .map_err(|err| assert_eq!(err.etype, LerrType::IncorrectParamCount));

        let _ = builtin_def(
            env,
            vec![
                Lval::Qexpr(vec![
                    Lval::Sym(String::from("a")),
                    Lval::Sym(String::from("b")),
                ]),
                Lval::Num(1_f64),
                Lval::Sym(String::from("+")),
                Lval::Sym(String::from("+")),
            ],
        )
        .map_err(|err| assert_eq!(err.etype, LerrType::IncorrectParamCount));
        let _ = builtin_def(
            env,
            vec![Lval::Qexpr(vec![Lval::Num(1_f64)]), Lval::Num(1_f64)],
        )
        .map_err(|err| assert_eq!(err.etype, LerrType::WrongType));
    }

    //(\ {a b} {* a b}) 1 2
    #[test]
    fn it_correctly_uses_lambda() {
        let env = &mut init_env();
        assert!(to_lambda(
            &builtin_lambda(
                env,
                vec![
                    Lval::Qexpr(vec![
                        Lval::Sym(String::from("a")),
                        Lval::Sym(String::from("b")),
                    ]),
                    Lval::Qexpr(vec![
                        Lval::Sym(String::from("+")),
                        Lval::Sym(String::from("a")),
                        Lval::Sym(String::from("b")),
                    ]),
                ]
            )
            .unwrap()
        )
        .is_some());

        let expr = Lval::Sexpr(vec![
            Lval::Sexpr(vec![
                Lval::Sym(String::from("\\")),
                Lval::Qexpr(vec![
                    Lval::Sym(String::from("a")),
                    Lval::Sym(String::from("b")),
                ]),
                Lval::Qexpr(vec![
                    Lval::Sym(String::from("+")),
                    Lval::Sym(String::from("a")),
                    Lval::Sym(String::from("b")),
                ]),
            ]),
            Lval::Num(2_f64),
            Lval::Num(2_f64),
        ]);
        assert_eq!(eval::eval(env, expr).unwrap(), Lval::Num(4_f64));
    }

    #[test]
    fn it_correctly_uses_ord() {
        let env = &mut init_env();
        assert_eq!(
            builtin_lt(env, vec![Lval::Num(1_f64), Lval::Num(2_f64)]).unwrap(),
            Lval::Num(1_f64)
        );
        assert_eq!(
            builtin_lt(env, vec![Lval::Num(2_f64), Lval::Num(1_f64)]).unwrap(),
            Lval::Num(0_f64)
        );

        assert_eq!(
            builtin_gt(env, vec![Lval::Num(1_f64), Lval::Num(2_f64)]).unwrap(),
            Lval::Num(0_f64)
        );
        assert_eq!(
            builtin_gt(env, vec![Lval::Num(2_f64), Lval::Num(1_f64)]).unwrap(),
            Lval::Num(1_f64)
        );

        assert_eq!(
            builtin_gte(env, vec![Lval::Num(1_f64), Lval::Num(2_f64)]).unwrap(),
            Lval::Num(0_f64)
        );
        assert_eq!(
            builtin_gte(env, vec![Lval::Num(2_f64), Lval::Num(1_f64)]).unwrap(),
            Lval::Num(1_f64)
        );
        assert_eq!(
            builtin_gte(env, vec![Lval::Num(2_f64), Lval::Num(2_f64)]).unwrap(),
            Lval::Num(1_f64)
        );

        assert_eq!(
            builtin_lte(env, vec![Lval::Num(1_f64), Lval::Num(2_f64)]).unwrap(),
            Lval::Num(1_f64)
        );
        assert_eq!(
            builtin_lte(env, vec![Lval::Num(2_f64), Lval::Num(1_f64)]).unwrap(),
            Lval::Num(0_f64)
        );
        assert_eq!(
            builtin_lte(env, vec![Lval::Num(2_f64), Lval::Num(2_f64)]).unwrap(),
            Lval::Num(1_f64)
        );
    }

    #[test]
    fn it_correctly_uses_if() {
        let env = &mut init_env();
        assert_eq!(
            builtin_if(
                env,
                vec![
                    Lval::Num(1_f64),
                    Lval::Qexpr(vec![Lval::Num(6_f64)]),
                    Lval::Qexpr(vec![Lval::Num(9_f64)])
                ]
            )
            .unwrap(),
            Lval::Num(6_f64)
        );
        assert_eq!(
            builtin_if(
                env,
                vec![
                    Lval::Num(0_f64),
                    Lval::Qexpr(vec![Lval::Num(6_f64)]),
                    Lval::Qexpr(vec![Lval::Num(9_f64)])
                ]
            )
            .unwrap(),
            Lval::Num(9_f64)
        );
    }
}
