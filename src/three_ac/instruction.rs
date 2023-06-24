use std::fmt;

use super::label::Label;
use super::operand::{self, Operand};

#[derive(Debug)]
pub struct Instruction {
    pub variant: Variant,
    pub set: Set,
    pub opdt: Operand,
    pub opm: Operand,
    pub opn: Operand,
}

impl Instruction {
    pub fn header_text(label: Label) -> Self {
        Self {
            variant: Variant::HeaderText(label),
            set: Set::T,
            opdt: Operand::new_null(),
            opm: Operand::new_null(),
            opn: Operand::new_null(),
        }
    }

    pub fn header_strings(strs: String) -> Self {
        Self {
            variant: Variant::HeaderStrings(strs),
            set: Set::T,
            opdt: Operand::new_null(),
            opm: Operand::new_null(),
            opn: Operand::new_null(),
        }
    }

    pub fn addr_assign(set: Set, opdt: Operand, opm: Operand) -> Self {
        Self {
            variant: Variant::AddrAssign,
            set,
            opdt,
            opm,
            opn: Operand::new_null(),
        }
    }

    pub fn assign(set: Set, opdt: Operand, opm: Operand) -> Self {
        Self {
            variant: Variant::Assign,
            set,
            opdt,
            opm,
            opn: Operand::new_null(),
        }
    }

    pub fn free(opdt: Operand) -> Self {
        Self {
            variant: Variant::Free,
            set: Set::T,
            opdt,
            opm: Operand::new_null(),
            opn: Operand::new_null(),
        }
    }

    pub fn get(set: Set, opdt: Operand) -> Self {
        Self {
            variant: Variant::Get,
            set,
            opdt,
            opm: Operand::new_null(),
            opn: Operand::new_null(),
        }
    }

    pub fn malloc(opdt: Operand, opm: Operand) -> Self {
        Self {
            variant: Variant::Malloc,
            set: Set::T,
            opdt,
            opm,
            opn: Operand::new_null(),
        }
    }

    pub fn put(set: Set, opdt: Operand, string_variant: bool) -> Self {
        Self {
            variant: if string_variant {
                Variant::PutS
            } else {
                Variant::Put
            },
            set,
            opdt,
            opm: Operand::new_null(),
            opn: Operand::new_null(),
        }
    }

    pub fn ret() -> Self {
        Self {
            variant: Variant::Ret,
            set: Set::T,
            opdt: Operand::new_null(),
            opm: Operand::new_null(),
            opn: Operand::new_null(),
        }
    }

    pub fn save(set: Set, opdt: Operand) -> Self {
        Self {
            variant: Variant::Save,
            set,
            opdt,
            opm: Operand::new_null(),
            opn: Operand::new_null(),
        }
    }

    pub fn load(set: Set, opdt: Operand, lit: String) -> Self {
        Self {
            variant: Variant::Load(lit),
            set,
            opdt,
            opm: Operand::new_null(),
            opn: Operand::new_null(),
        }
    }

    pub fn address(opdt: Operand, opm: Operand) -> Self {
        Self {
            variant: Variant::Address,
            set: Set::T,
            opdt,
            opm,
            opn: Operand::new_null(),
        }
    }

    pub fn dereference(set: Set, opdt: Operand, opm: Operand) -> Self {
        Self {
            variant: Variant::Dereference,
            set,
            opdt,
            opm,
            opn: Operand::new_null(),
        }
    }

    pub fn reference(opdt: Operand, opm: Operand) -> Self {
        Self {
            variant: Variant::Reference,
            set: Set::T,
            opdt,
            opm,
            opn: Operand::new_null(),
        }
    }

    pub fn negate(set: Set, opdt: Operand, opm: Operand) -> Self {
        Self {
            variant: Variant::Negate,
            set,
            opdt,
            opm,
            opn: Operand::new_null(),
        }
    }

    pub fn cast(set: Set, opdt: Operand, opm: Operand) -> Self {
        Self {
            variant: Variant::Cast,
            set,
            opdt,
            opm,
            opn: Operand::new_null(),
        }
    }

    pub fn label(label: Label) -> Self {
        Self {
            variant: Variant::Label(label),
            set: Set::T,
            opdt: Operand::new_null(),
            opm: Operand::new_null(),
            opn: Operand::new_null(),
        }
    }

    pub fn jump(label: Label) -> Self {
        Self {
            variant: Variant::Jump(label),
            set: Set::T,
            opdt: Operand::new_null(),
            opm: Operand::new_null(),
            opn: Operand::new_null(),
        }
    }

    pub fn call(label: Label, args: Vec<Operand>, set: Set, opdt: Operand) -> Self {
        Self {
            variant: Variant::Call(label, args),
            set,
            opdt,
            opm: Operand::new_null(),
            opn: Operand::new_null(),
        }
    }

    pub fn alloc(bytes: i32) -> Self {
        Self {
            variant: Variant::Alloc(bytes.unsigned_abs()),
            set: Set::T,
            opdt: Operand::new_null(),
            opm: Operand::new_null(),
            opn: Operand::new_null(),
        }
    }

    pub fn spill_registers() -> Self {
        Self {
            variant: Variant::SpillRegisters,
            set: Set::T,
            opdt: Operand::new_null(),
            opm: Operand::new_null(),
            opn: Operand::new_null(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Variant {
    HeaderText(Label),
    HeaderStrings(String),

    AddrAssign,
    Assign,
    Free,
    Get,
    Malloc,
    Put,
    PutS,
    Ret,
    Save,
    Load(String),

    Address,
    Dereference,
    Reference,

    Plus,
    Minus,
    Times,
    Divide,
    Negate,

    Cast,

    Equal(Label),
    NotEqual(Label),
    Less(Label),
    LessEqual(Label),
    Greater(Label),
    GreaterEqual(Label),

    Label(Label),
    Jump(Label),
    Call(Label, Vec<Operand>),

    Alloc(u32),
    SpillRegisters,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Set {
    T,
    F,
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.variant {
            Variant::HeaderText(l) => writeln!(f, "HEADER TEXT | JR {l}"),
            Variant::HeaderStrings(s) => write!(f, "HEADER STRINGS\n{s}"),

            Variant::AddrAssign => writeln!(f, "{} <= {}", self.opdt, self.opm),
            Variant::Assign => writeln!(f, "{} = {}", self.opdt, self.opm),
            Variant::Free => writeln!(f, "FREE {}", self.opdt),
            Variant::Get => writeln!(f, "GET {}", self.opdt),
            Variant::Malloc => writeln!(f, "{} = MALLOC {}", self.opdt, self.opm),
            Variant::Put | Variant::PutS => writeln!(f, "PUT {}", self.opdt),
            Variant::Ret => writeln!(f, "RET"),
            Variant::Save => writeln!(f, "Save {}", self.opdt),
            Variant::Load(v) => writeln!(f, "{} := {v}", self.opdt),

            Variant::Address => writeln!(f, "{} = %[{}]", self.opdt, self.opm),
            Variant::Dereference => writeln!(f, "{} = *[{}]", self.opdt, self.opm),
            Variant::Reference => writeln!(f, "{} = &[{}]", self.opdt, self.opm),

            Variant::Plus => {
                writeln!(f, "{} = {} + {}", self.opdt, self.opm, self.opn)
            }
            Variant::Minus => {
                writeln!(f, "{} = {} - {}", self.opdt, self.opm, self.opn)
            }
            Variant::Times => {
                writeln!(f, "{} = {} * {}", self.opdt, self.opm, self.opn)
            }
            Variant::Divide => {
                writeln!(f, "{} = {} / {}", self.opdt, self.opm, self.opn)
            }
            Variant::Negate => writeln!(f, "{} = NEG {}", self.opdt, self.opm),

            Variant::Cast => writeln!(f, "{} = CAST {}", self.opdt, self.opm),

            Variant::Equal(l) => writeln!(f, "{} == {} ? else J {l}", self.opm, self.opn),
            Variant::NotEqual(l) => writeln!(f, "{} != {} ? else J {l}", self.opm, self.opn),
            Variant::Less(l) => writeln!(f, "{} < {} ? else J {l}", self.opm, self.opn),
            Variant::LessEqual(l) => writeln!(f, "{} <= {} ? else J {l}", self.opm, self.opn),
            Variant::Greater(l) => writeln!(f, "{} > {} ? else J {l}", self.opm, self.opn),
            Variant::GreaterEqual(l) => writeln!(f, "{} >= {} ? else J {l}", self.opm, self.opn),

            Variant::Label(l) => {
                writeln!(f, "{l}:")
            }
            Variant::Jump(l) => {
                writeln!(f, "J {l}")
            }
            Variant::Call(l, args) => {
                let mut arg = String::new();
                for a in args {
                    if arg.is_empty() {
                        arg = format!("{}", a);
                    } else {
                        arg = format!("{}, {}", arg, a);
                    }
                }
                match self.opdt.variant {
                    operand::Variant::Null => writeln!(f, "{l}({})", arg),
                    _ => writeln!(f, "{} = {l}({})", self.opdt, arg),
                }
            }

            Variant::Alloc(u) => writeln!(f, "ALLOC {u}"),
            Variant::SpillRegisters => writeln!(f, "SPILL REGISTERS"),
        }
    }
}
