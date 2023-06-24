use super::ctype::CType;

use crate::error::Error;

#[derive(Clone, Debug)]
pub enum Entry {
    Symbol {
        ctype: CType,
        address: i32,
        symtype: SymbolType,
    },
    Function {
        ctype: CType,
        address: i32,
        arguments: Vec<CType>,
        scope: usize,
    },
}

#[derive(Clone, Debug)]
pub enum SymbolType {
    Global,
    Local,
    Argument,
    Str(String),
}

impl Entry {
    pub fn ctype(&self) -> CType {
        match self {
            Entry::Symbol { ctype, .. } => ctype.clone(),
            Entry::Function { ctype, .. } => ctype.clone(),
        }
    }

    pub fn address(&self) -> i32 {
        match self {
            Entry::Symbol { address, .. } => *address,
            Entry::Function { address, .. } => *address,
        }
    }

    pub fn arguments(&self) -> Result<Vec<CType>, Error> {
        match self {
            Entry::Symbol { .. } => Err(Error::SymTable(String::from(
                "arguments: symbol do not have arguments",
            ))),
            Entry::Function { arguments, .. } => Ok(arguments.to_vec()),
        }
    }

    pub fn scope(&self) -> usize {
        match self {
            Entry::Symbol { .. } => 0,
            Entry::Function { scope, .. } => *scope,
        }
    }

    pub fn set_scope(self, scope: usize) -> Self {
        match self {
            Entry::Symbol {
                ctype,
                address,
                symtype,
            } => Self::Symbol {
                ctype,
                address,
                symtype,
            },
            Entry::Function {
                ctype,
                address,
                arguments,
                ..
            } => Self::Function {
                ctype,
                address,
                arguments,
                scope,
            },
        }
    }
}
