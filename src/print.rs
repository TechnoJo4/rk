use std::fmt;

use crate::value::Value;

impl fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int(i) => write!(f, "{}", i),
            Self::List(vec) => {
                write!(f, "(")?;
                for (i, v) in vec.iter().enumerate() {
                    if i != 0 { write!(f, ";")?; }
                    write!(f, "{}", v)?;
                }
                write!(f, ")")
            }
        }
    }
}
