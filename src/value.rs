use core::fmt;
use std::rc::Rc;

pub const VERBS: &[u8] = "!#$%&*+,-.:<=>?@^_|~".as_bytes();

#[derive(Clone, Copy)]
pub enum Verb {
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T,
}

impl Verb {
    pub const fn from_char(chr: char) -> Option<Verb> {
        match chr {
            // If ever anything is changed here, also change in parse.rs
            '!' => Some(Verb::A),
            '#' => Some(Verb::B),
            '$' => Some(Verb::C),
            '%' => Some(Verb::D),
            '&' => Some(Verb::E),
            '*' => Some(Verb::F),
            '+' => Some(Verb::G),
            ',' => Some(Verb::H),
            '-' => Some(Verb::I),
            '.' => Some(Verb::J),
            ':' => Some(Verb::K),
            '<' => Some(Verb::L),
            '=' => Some(Verb::M),
            '>' => Some(Verb::N),
            '?' => Some(Verb::O),
            '@' => Some(Verb::P),
            '^' => Some(Verb::Q),
            '_' => Some(Verb::R),
            '|' => Some(Verb::S),
            '~' => Some(Verb::T),
            _ => None
        }
    }

    pub const fn to_char(&self) -> char {
        VERBS[*self as usize] as char
    }
}

impl fmt::Display for Verb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

impl fmt::Debug for Verb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Verb({})", self.to_char())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Adverb {
    S, // /
    B, // \
    Q, // '
    SC,// /:
    BC,// \:
    QC,// ':
    WB,//  \
    WQ,//  '
}

impl fmt::Display for Adverb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Adverb::S => "/",
            Adverb::B => "\\",
            Adverb::Q => "'",
            Adverb::SC => "/:",
            Adverb::BC => "\\:",
            Adverb::QC => "':",
            Adverb::WB => " \\",
            Adverb::WQ => " '",
        })
    }
}

#[derive(Debug)]
pub enum Value {
    Int(i64),
    List(Rc<Vec<Value>>),
}

impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            Self::Int(i) => Self::Int(*i),
            Self::List(rc) => Self::List(rc.clone()),
        }
    }
}
