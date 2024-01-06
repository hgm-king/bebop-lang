use crate::lisp::{builtin::init_builtins, Lval};
use std::collections::HashMap;

#[derive(Clone)]
pub struct Lenv {
    head: LinkedEnv,
}

type LinkedEnv = Option<Box<Env>>;
pub type Lookup = HashMap<String, Lval>;

#[derive(Clone, Debug)]
pub struct Env {
    lookup: Lookup,
    parent: LinkedEnv,
}

impl Lenv {
    pub fn new() -> Self {
        Lenv { head: None }
    }
}

impl Lenv {
    pub fn push(&mut self, lookup: Lookup) {
        let new_env = Box::new(Env {
            lookup,
            parent: self.head.take(),
        });

        self.head = Some(new_env);
    }

    pub fn pop(&mut self) -> Option<Lookup> {
        self.head.take().map(|env| {
            self.head = env.parent;
            env.lookup
        })
    }

    pub fn peek(&self) -> Option<&Lookup> {
        self.head.as_ref().map(|env| &env.lookup)
    }

    pub fn peek_mut(&mut self) -> Option<&mut Lookup> {
        self.head.as_mut().map(|env| &mut env.lookup)
    }

    pub fn iter(&self) -> Iter<'_> {
        Iter {
            next: self.head.as_deref(),
        }
    }

    pub fn insert(&mut self, key: &str, lval: Lval) {
        self.peek_mut()
            .map(|node| node.insert(key.to_owned(), lval));
    }

    pub fn insert_last(&mut self, key: &str, lval: Lval) {
        let mut i = self.head.as_mut();

        while let Some(env) = i {
            i = env.parent.as_mut();
            if let None = i {
                env.lookup.insert(key.to_owned(), lval.clone());
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<Lval> {
        let mut i = self.iter();

        while let Some(env) = i.next() {
            if let Some(v) = env.get(key) {
                return Some(v.clone());
            }
        }

        None
    }
}

impl Drop for Lenv {
    fn drop(&mut self) {
        let mut cur_link = self.head.take();
        while let Some(mut boxed_env) = cur_link {
            cur_link = boxed_env.parent.take();
        }
    }
}

pub struct Iter<'a> {
    next: Option<&'a Env>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Lookup;
    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|env| {
            self.next = env.parent.as_deref();
            &env.lookup
        })
    }
}

pub fn init_env() -> Lenv {
    let mut env = Lenv::new();
    env.push(Lookup::new());
    init_builtins(&mut env);
    env
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_nests_properly() {
        let mut env = Lenv::new();
        env.push(Lookup::new());
        env.insert("abc", Lval::Num(1_f64));
        env.insert("def", Lval::Num(2_f64));

        env.push(Lookup::new());
        env.insert("abc", Lval::Num(3_f64));
        env.insert("ghi", Lval::Num(4_f64));

        assert_eq!(env.get("def").unwrap().to_owned(), Lval::Num(2_f64));
        assert_eq!(env.get("abc").unwrap().to_owned(), Lval::Num(3_f64));
        env.pop();

        assert_eq!(env.get("abc").unwrap().to_owned(), Lval::Num(1_f64));
        assert_eq!(env.get("def").unwrap().to_owned(), Lval::Num(2_f64));
        assert_eq!(env.get("ghi"), None);
    }

    #[test]
    fn it_inserts_last() {
        let mut env = Lenv::new();
        env.push(Lookup::new());
        env.insert("abc", Lval::Num(1_f64));
        env.insert_last("def", Lval::Num(2_f64));

        env.push(Lookup::new());
        env.insert("abc", Lval::Num(3_f64));
        env.insert_last("jkl", Lval::Num(5_f64));

        assert_eq!(env.get("def").unwrap().to_owned(), Lval::Num(2_f64));
        assert_eq!(env.get("abc").unwrap().to_owned(), Lval::Num(3_f64));
        assert_eq!(env.get("jkl").unwrap().to_owned(), Lval::Num(5_f64));

        env.pop();

        assert_eq!(env.get("jkl").unwrap().to_owned(), Lval::Num(5_f64));
        assert_eq!(env.get("abc").unwrap().to_owned(), Lval::Num(1_f64));
    }

    #[test]
    fn it_grabs_from_higher_environments() {
        let mut env = Lenv::new();
        env.push(Lookup::new()); // base
        env.insert("a", Lval::Num(1_f64));
        env.insert_last("b", Lval::Num(2_f64));

        assert_eq!(env.get("a").unwrap().to_owned(), Lval::Num(1_f64));
        assert_eq!(env.get("b").unwrap().to_owned(), Lval::Num(2_f64));

        env.push(Lookup::new()); // 2nd
        env.insert("f", Lval::Num(3_f64));

        assert_eq!(env.get("a").unwrap().to_owned(), Lval::Num(1_f64));
        assert_eq!(env.get("b").unwrap().to_owned(), Lval::Num(2_f64));
        assert_eq!(env.get("f").unwrap().to_owned(), Lval::Num(3_f64));

        env.push(Lookup::new()); // 3rd
        env.insert("g", Lval::Num(4_f64));

        assert_eq!(env.get("a").unwrap().to_owned(), Lval::Num(1_f64));
        assert_eq!(env.get("b").unwrap().to_owned(), Lval::Num(2_f64));
        assert_eq!(env.get("f").unwrap().to_owned(), Lval::Num(3_f64));
        assert_eq!(env.get("g").unwrap().to_owned(), Lval::Num(4_f64));

        env.pop();
        assert_eq!(env.get("a").unwrap().to_owned(), Lval::Num(1_f64));
        assert_eq!(env.get("b").unwrap().to_owned(), Lval::Num(2_f64));
        assert_eq!(env.get("f").unwrap().to_owned(), Lval::Num(3_f64));

        env.pop();
        assert_eq!(env.get("a").unwrap().to_owned(), Lval::Num(1_f64));
        assert_eq!(env.get("b").unwrap().to_owned(), Lval::Num(2_f64));
    }
}
