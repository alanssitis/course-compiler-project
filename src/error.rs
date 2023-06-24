use std::fmt;
use std::io;

use crate::parser::Rule;

pub enum Error {
    Io(Box<io::Error>),
    Parse(Box<pest::error::Error<Rule>>),
    SymTable(String),
    ThreeAC(String),
    RegAlloc(String),
    PairsNext,
    Type,
    Other(String),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::Io(Box::new(e))
    }
}

impl From<pest::error::Error<Rule>> for Error {
    fn from(e: pest::error::Error<Rule>) -> Self {
        Self::Parse(Box::new(e))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{e}"),
            Self::Parse(e) => write!(f, "{e}"),
            Self::SymTable(s) => write!(f, "{s}"),
            Self::ThreeAC(s) => write!(f, "{s}"),
            Self::RegAlloc(s) => write!(f, "{s}"),
            Self::PairsNext => write!(f, "failed to get next sub pair"),
            Self::Type => write!(f, "TYPE ERROR"),
            Self::Other(s) => write!(f, "{s}"),
        }
    }
}
