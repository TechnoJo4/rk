use std::{mem, rc::Rc};

use crate::value::{self, Value, Verb};

#[repr(u8)]
#[derive(PartialEq)]
enum Type {
    N = 0, M = 1, D = 2, A = 3
}

const STRENGTHS: [u8; 16] = [
    1, 0, 3, 4,//N
    2, 1, 0, 4,//M
    2, 1, 1, 4,//D
    0, 0, 0, 0,//A
];//N  M  D  A

fn strength(l: Type, r: Type) -> u8 {
    STRENGTHS[(l as usize)*4 + (r as usize)]
}

#[derive(Debug)]
pub enum Node {
    Literal(Value),                         //1 2 3
    Var(Vec<char>),                         //abc
    Assign(Box<Vec<char>>, Box<Node>),      //abc:x
    Verb(value::Verb),                      //+
    Monad(value::Verb),                     //+:
    Adverb(value::Adverb),                  //\
    List(Vec<Node>),                        //(a;b;c)
    Fun(Vec<Node>),                         //{a;b;c}
    Progn(Vec<Node>),                       //[a;b;c]
    Apply(Box<Node>, Box<Node>),            //+x
    Apply2(Box<Node>, Box<Node>, Box<Node>),//x+y
    WithL { l: Box<Node>, f: Box<Node> },   //x+
    ApplyA { v: Box<Node>, a: Box<Node> },  //+/
    ApplyN(Box<Node>, Vec<Node>),           //f[a;b;c]
    Compose(Box<Node>, Box<Node>),          //++
    Noun(Box<Node>)                         //(f)
}

impl Node {
    pub(self) fn t(&self) -> Type {
        match &*self {
            Node::Literal(_) => Type::N,
            Node::Var(_) => Type::N,
            Node::Assign(_, _) => Type::N,
            Node::Verb(_) => Type::D,
            Node::Monad(_) => Type::M,
            Node::Adverb(_) => Type::A,
            Node::List(_) => Type::N,
            Node::Fun(_) => Type::N,
            Node::Progn(_) => Type::N,
            Node::Apply(_, _) => Type::N,
            Node::Apply2(_, _, _) => Type::N,
            Node::WithL { l: _, f: _ } => Type::M,
            Node::ApplyA { v: _, a: _ } => Type::D,
            Node::ApplyN(_, _) => Type::N,
            Node::Compose(_, _) => Type::M,
            Node::Noun(_) => Type::N,
        }
    }

    pub(self) fn nominalize(self) -> Self {
        if self.t() == Type::N {
            self
        } else {
            Self::Noun(Box::new(self))
        }
    }
}


struct HoledTape<T> {
    v: Vec<Option<T>>,
    d: Vec<usize>,
    i: usize
}

impl<T> HoledTape<T> {
    pub fn new(elems: Vec<T>) -> HoledTape<T> {
        let size = elems.len();
        assert!(size >= 2);

        HoledTape {
            v: elems.into_iter().map(|elem| Some(elem)).collect(),
            d: vec![1; size-1],
            i: size-2
        }
    }

    fn ir(&self) -> usize {
        self.i + self.d[self.i]
    }

    /// Bind the next element into the current, leaving a hole
    pub fn bind(&mut self, binder: fn(T, T) -> T) {
        let ir = self.ir();
        let c = mem::take(&mut self.v[self.i]).unwrap();
        let r = mem::take(&mut self.v[ir]).unwrap();

        self.v[self.i] = Some(binder(c, r));

        self.d[self.i] += *self.d.get(ir).unwrap_or(&1);
        if self.ir() >= self.v.len() {
            self.go()
        }
    }

    /// Bind everything left-to-right
    pub fn fold(mut self, binder: fn(T, T) -> T) -> T {
        self.i = 0;
        let mut c = mem::take(&mut self.v[0]).unwrap();

        let mut ir = 0;
        while ir < self.d.len() {
            ir += self.d[ir];

            if ir >= self.v.len() { break; }
            let r = mem::take(&mut self.v[ir]).unwrap();
            c = binder(c, r)
        }
        c
    }

    pub fn has_left(&self) -> bool {
        self.i != 0
    }

    pub fn go(&mut self) {
        assert!(self.has_left(), "cannot move HoledTape past start");
        self.i -= 1
    }

    pub fn left(&self) -> Option<&T> {
        self.v.get(self.i - 1).map(|o| o.as_ref().unwrap())
    }

    pub fn center(&self) -> &T {
        self.v[self.i].as_ref().unwrap()
    }

    pub fn right(&self) -> &T {
        self.v[self.ir()].as_ref().unwrap()
    }

    pub fn map_lc<U>(&self, f: fn(&T) -> U) -> (U, U) {
        (f(self.left().unwrap()), f(self.center()))
    }

    pub fn map_cr<U>(&self, f: fn(&T) -> U) -> (U, U) {
        (f(self.center()), f(self.right()))
    }
}

struct Parser {
    s: Vec<char>,
    si: usize
}

impl Parser {
    fn peek(&self) -> Option<char> {
        self.s.get(self.si).copied()
    }

    fn advance(&mut self) {
        self.si += 1
    }

    fn ws(&self, c: char) -> bool {
        match c {
            ' ' | '\t' | '\r' | '\n' => true,
            _ => false
        }
    }

    fn was_ws(&self) -> bool {
        self.si == 0 || self.ws(self.s[self.si-1])
    }

    fn is_ws(&self) -> bool {
        self.si <= self.s.len() && self.ws(self.s[self.si-1])
    }

    fn skip_ws(&mut self) {
        while self.is_ws() {
            self.advance()
        }
    }

    fn name(&mut self) -> Vec<char> {
        let mut vec = Vec::new();
        vec.push(self.s[self.si]);
        self.advance();
        loop {
            match self.peek() {
                Some('a'..'z') | Some('A'..'Z') | Some('0'..'9') => {
                    vec.push(self.s[self.si]);
                    self.advance();
                },
                _ => break
            }
        }
        vec
    }

    fn nums(&mut self) -> Value {
        let v = self.s[self.si].to_digit(10).unwrap() as i64;
        self.advance();
        Value::Int(v)
    }

    fn verb(&mut self, v: Verb) -> Result<Node, String> {
        self.advance();
        if self.peek() == Some(':') {
            self.advance();
            Ok(Node::Monad(v))
        } else {
            Ok(Node::Verb(v))
        }
    }

    fn item(&mut self) -> Result<Node, String> {
        match self.peek() {
            Some('(') => {
                self.advance();
                let mut vec = self.obj(')')?;
                if vec.len() == 1 {
                    Ok(vec.pop().unwrap().nominalize())
                } else {
                    Ok(Node::List(vec))
                }
            },
            Some('[') => {
                self.advance();
                Ok(Node::Progn(self.obj(']')?))
            },
            Some('{') => {
                self.advance();
                /*if self.peek() == Some('[') {
                    self.advance();
                    let mut names: Vec<Vec<char>> = Vec::new();
                    loop {
                        if let Node::Var(name) = self.item()? {
                            names.push(name)
                        } else {
                            return Err("excepted <name> in args (after '{[')".to_owned())
                        }
                        match self.peek() {
                            Some(']') => break,
                            Some(';') => self.advance(),
                            Some(c) => return Err(format!("unexpected character '{}'", c)),
                            None => return Err(format!("unexpected <eof>"))
                        }
                    }
                }*/

                Ok(Node::Fun(self.obj('}')?))
            },
            // If ever anything is changed here, also change in value.rs
            Some('!') => self.verb(Verb::A),
            Some('#') => self.verb(Verb::B),
            Some('$') => self.verb(Verb::C),
            Some('%') => self.verb(Verb::D),
            Some('&') => self.verb(Verb::E),
            Some('*') => self.verb(Verb::F),
            Some('+') => self.verb(Verb::G),
            Some(',') => self.verb(Verb::H),
            Some('-') => self.verb(Verb::I),
            Some('.') => self.verb(Verb::J),
            Some(':') => self.verb(Verb::K),
            Some('<') => self.verb(Verb::L),
            Some('=') => self.verb(Verb::M),
            Some('>') => self.verb(Verb::N),
            Some('?') => self.verb(Verb::O),
            Some('@') => self.verb(Verb::P),
            Some('^') => self.verb(Verb::Q),
            Some('_') => self.verb(Verb::R),
            Some('|') => self.verb(Verb::S),
            Some('~') => self.verb(Verb::T),
            Some(c) if c.is_digit(10) => {
                Ok(Node::Literal(self.nums()))
            },
            Some(c) if c.is_ascii_alphabetic() => Ok(Node::Var(self.name())),
            Some(c) => Err(format!("unexpected character '{}'", c)),
            None => Err(format!("unexpected <eof>"))
        }
    }

    fn bind(left: Node, right: Node) -> Node {
        match (left.t(), right.t()) {
            (Type::N, Type::D) => Node::WithL { l: Box::new(left), f: Box::new(right) },
            (Type::N, Type::A) => Node::WithL { l: Box::new(left), f: Box::new(right) },
            (Type::M, Type::A) => Node::ApplyA { v: Box::new(left), a: Box::new(right) },
            (Type::D, Type::A) => Node::ApplyA { v: Box::new(left), a: Box::new(right) },
            (Type::D, Type::M) => Node::Compose(Box::new(left), Box::new(right)),
            (Type::D, Type::D) => Node::Compose(Box::new(left), Box::new(right)),
            (Type::A, Type::M) => Node::Compose(Box::new(left), Box::new(right)),
            (Type::A, Type::D) => Node::Compose(Box::new(left), Box::new(right)),
            (Type::M, Type::M) => Node::Compose(Box::new(left), Box::new(right)),
            (Type::M, Type::N) => match left {
                Node::WithL { l, f } => match (*l, &*f) {
                    (Node::Var(name), Node::Verb(Verb::K)) => Node::Assign(Box::new(name), Box::new(right)),
                    (l, _) => Node::Apply2(f, Box::new(l), Box::new(right))
                },
                _ => match right {
                    Node::Progn(vec) => Node::ApplyN(Box::new(left), vec),
                    _ => Node::Apply(Box::new(left), Box::new(right))
                }
            },
            (_, Type::N) => match right {
                Node::Progn(vec) => Node::ApplyN(Box::new(left), vec),
                _ => Node::Apply(Box::new(left), Box::new(right))
            },
            (_, _) => Node::Apply(Box::new(left), Box::new(right)),
        }
    }

    fn strength((l, r): (Type, Type)) -> u8 {
        strength(l, r)
    }

    fn expr(&mut self, close: Option<char>) -> Result<Node, String> {
        let mut items = Vec::<Node>::new();
        loop {
            let c = self.peek();
            if c == Some(';') || c == Some('\n') {
                self.advance();
                break
            }
            if c == close { break }
            items.push(self.item()?)
        }

        match items.len() {
            0 => Err(format!("empty expression")),
            1 => Ok(items.pop().unwrap()),
            _ => {
                let mut tape = HoledTape::new(items);

                while tape.has_left() {
                    let lc = Parser::strength(tape.map_lc(Node::t));
                    let cr = Parser::strength(tape.map_cr(Node::t));
                    if lc > cr {
                        tape.go();
                        continue
                    }

                    tape.bind(Parser::bind)
                }

                Ok(tape.fold(Parser::bind))
            }
        }
    }

    fn obj(&mut self, close: char) -> Result<Vec<Node>, String> {
        let mut vec = Vec::<Node>::new();
        loop {
            let chr = self.peek();
            if chr.is_none() {
                return Err(format!("unexpected <eof>, expected '{}'", close))
            }
            if chr.unwrap() == close {
                self.advance();
                break
            }

            vec.push(self.expr(Some(close))?)
        }
        Ok(vec)
    }

    fn file(&mut self) -> Result<Node, String> {
        let mut vec = Vec::<Node>::new();
        while self.peek().is_some() {
            vec.push(self.expr(None)?)
        }
        Ok(Node::Progn(vec))
    }
}

pub fn parse(src: String) -> Result<Node, String> {
    Parser {
        s: src.chars().collect(),
        si: 0
    }.file()
}
