use crate::error::Error;
use crate::symtable::CType;

#[derive(Debug)]
pub enum Node {
    // Empty
    Empty,

    // Statements
    Assign {
        ctype: CType,
        lhs: Box<Self>,
        rhs: Box<Self>,
    },
    Free {
        expr: Box<Self>,
    },
    Malloc {
        ctype: CType,
        expr: Box<Self>,
    },
    Read {
        ctype: CType,
        var: Box<Self>,
    },
    Return {
        ctype: CType,
        function: i32,
        expr: Box<Self>,
    },
    StatementList {
        statements: Vec<Self>,
    },
    Write {
        ctype: CType,
        expr: Box<Self>,
    },

    // Constructs
    IfElse {
        cond: Box<Self>,
        lhs: Box<Self>, // then
        rhs: Box<Self>, // else
    },
    While {
        cond: Box<Self>,
        statements: Box<Self>,
    },

    // Operations
    BinaryOp {
        ctype: CType,
        op: BinOp,
        lhs: Box<Self>,
        rhs: Box<Self>,
    },
    ConditionalOp {
        ctype: CType,
        op: CondOp,
        lhs: Box<Self>,
        rhs: Box<Self>,
    },
    UnaryOp {
        ctype: CType,
        expr: Box<Self>,
    },
    Cast {
        ctype: CType,
        expr: Box<Self>,
    },
    Address {
        ctype: CType,
        expr: Box<Self>,
    },
    Dereference {
        ctype: CType,
        expr: Box<Self>,
    },
    Reference {
        ctype: CType,
        expr: Box<Self>,
    },

    // Function
    Function {
        ident: String,
        scope: usize,
        statements: Box<Self>,
    },
    Call {
        ctype: CType,
        ident: String,
        scope: usize,
        arguments: Vec<Self>,
    },

    // Literals
    FloatLit {
        ctype: CType,
        val: f32,
    },
    IntLit {
        ctype: CType,
        val: i32,
    },
    Var {
        ctype: CType,
        ident: String,
        scope: usize,
    },
}

impl Node {
    pub fn ctype(&self) -> CType {
        match self {
            Self::Assign { ctype, .. } => ctype.clone(),
            Self::Malloc { ctype, .. } => ctype.clone(),
            Self::Read { ctype, .. } => ctype.clone(),
            Self::Return { ctype, .. } => ctype.clone(),
            Self::Write { ctype, .. } => ctype.clone(),
            Self::BinaryOp { ctype, .. } => ctype.clone(),
            Self::UnaryOp { ctype, .. } => ctype.clone(),
            Self::Cast { ctype, .. } => ctype.clone(),
            Self::Address { ctype, .. } => ctype.clone(),
            Self::Dereference { ctype, .. } => ctype.clone(),
            Self::Reference { ctype, .. } => ctype.clone(),
            Self::Call { ctype, .. } => ctype.clone(),
            Self::FloatLit { ctype, .. } => ctype.clone(),
            Self::IntLit { ctype, .. } => ctype.clone(),
            Self::Var { ctype, .. } => ctype.clone(),
            _ => CType::Void,
        }
    }

    pub fn strip_ctype(&self) -> Result<CType, Error> {
        match self {
            Self::Address { expr, .. } => expr.strip_ctype()?.dereference(),
            _ => Ok(self.ctype()),
        }
    }

    pub fn set_ctype(self, new: &CType) -> Result<Self, Error> {
        if self.ctype() == *new {
            return Ok(self);
        }
        if !(self.ctype().is_mutable() && new.is_mutable()) {
            return Err(Error::Type);
        }

        match self {
            Self::BinaryOp { op, lhs, rhs, .. } => Ok(Self::BinaryOp {
                ctype: new.clone(),
                op,
                lhs: Box::new(lhs.set_ctype(new)?),
                rhs: Box::new(rhs.set_ctype(new)?),
            }),
            Self::UnaryOp { expr, .. } => Ok(Self::UnaryOp {
                ctype: new.clone(),
                expr: Box::new(expr.set_ctype(new)?),
            }),
            Self::FloatLit { val, .. } => {
                if !matches!(new, CType::Float) {
                    return Err(Error::Type);
                }
                Ok(Self::FloatLit {
                    ctype: new.clone(),
                    val,
                })
            }
            Self::IntLit { val, .. } => {
                if !matches!(new, CType::Float | CType::Ptr(_)) {
                    return Err(Error::Type);
                }
                Ok(Self::IntLit {
                    ctype: new.clone(),
                    val,
                })
            }
            Self::Malloc { expr, .. } => {
                if !matches!(new, CType::Ptr(_)) {
                    return Err(Error::Type);
                }
                Ok(Self::Malloc {
                    ctype: new.clone(),
                    expr,
                })
            }
            _ => Err(Error::Type),
        }
    }

    pub fn cast(self, ctype: &CType) -> Self {
        Self::Cast {
            ctype: ctype.clone(),
            expr: Box::new(self),
        }
    }
}

#[derive(Debug)]
pub enum BinOp {
    Plus,
    Minus,
    Times,
    Divide,
}

#[derive(Debug)]
pub enum CondOp {
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}
