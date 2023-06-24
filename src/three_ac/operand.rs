use std::fmt;

use crate::error::Error;
use crate::symtable::{self, CType, SymbolType};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Operand {
    pub variant: Variant,
    pub otype: Type,
}

impl Operand {
    pub fn new_null() -> Self {
        Self {
            variant: Variant::Null,
            otype: Type::T,
        }
    }

    pub fn from_symbol(ctype: &CType, symbol: symtable::Entry) -> Result<Self, Error> {
        Ok(Self {
            variant: match symbol {
                symtable::Entry::Symbol {
                    address, symtype, ..
                } => match symtype {
                    SymbolType::Global => Variant::Global(address),
                    SymbolType::Str(_) => Variant::Str(address),
                    SymbolType::Local | SymbolType::Argument => Variant::Local(address),
                },
                symtable::Entry::Function { .. } => {
                    return Err(Error::ThreeAC(String::from(
                        "from_var: encountered function instead of var",
                    )))
                }
            },
            otype: Type::from_ctype(ctype),
        })
    }

    pub fn is_spillable(&self) -> bool {
        !matches!(self.variant, Variant::Null)
    }

    pub fn is_variable(&self) -> bool {
        matches!(self.variant, Variant::Global(_) | Variant::Local(_))
    }
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.variant {
            Variant::Global(a) => write!(f, "$G<0x{a:08x}>"),
            Variant::Str(a) => write!(f, "S<0x{a:08x}>"),
            Variant::Local(o) => write!(f, "L<{o}>"),

            Variant::Temp(n) => write!(f, "$T<{n}>"),
            Variant::TempFloat(n) => write!(f, "$TF<{n}>"),

            Variant::Null => write!(f, ""),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Variant {
    Global(i32),
    Str(i32),
    Local(i32),

    Temp(u32),
    TempFloat(u32),

    Null,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Type {
    T,
    F,
}

impl Type {
    pub fn from_ctype(ctype: &CType) -> Self {
        match ctype {
            CType::Int | CType::Str => Type::T,
            CType::Float => Type::F,
            CType::Ptr(_) => Type::T,
            CType::Void => unreachable!("from_ctype: encountered a void operand"),
        }
    }
}
