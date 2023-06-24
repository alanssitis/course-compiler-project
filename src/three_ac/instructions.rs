use std::collections::VecDeque;
use std::fmt;

use super::instruction::{self, Instruction};
use super::label::Label;
use super::operand::{self, Operand};

use crate::ast::{self, BinOp, CondOp};
use crate::error::Error;
use crate::symtable::{CType, SymTable};

pub struct Count {
    pub regular: u32,
    pub float: u32,
    pub label: u32,
}

impl Count {
    fn reset(&mut self) {
        self.regular = 0;
        self.float = 0;
    }
}

#[derive(Debug)]
pub struct Instructions {
    pub instructions: VecDeque<Instruction>,
    pub tmp: Option<Operand>,
}

impl Instructions {
    pub fn from_ast(
        node: ast::Node,
        count: &mut Count,
        symtable: &mut SymTable,
    ) -> Result<Self, Error> {
        match node {
            ast::Node::Assign { ctype, lhs, rhs } => {
                Self::from_assign(ctype, *lhs, *rhs, count, symtable)
            }
            ast::Node::Free { expr } => Self::from_free(*expr, count, symtable),
            ast::Node::Malloc { ctype, expr } => Self::from_malloc(ctype, *expr, count, symtable),
            ast::Node::Read { ctype, var } => Self::from_read(ctype, *var, count, symtable),
            ast::Node::Return {
                ctype,
                expr,
                function,
            } => Self::from_return(ctype, *expr, function, count, symtable),
            ast::Node::StatementList { statements } => {
                Self::from_statement_list(statements, count, symtable)
            }
            ast::Node::Write { ctype, expr } => Self::from_write(ctype, *expr, count, symtable),
            ast::Node::IfElse { cond, lhs, rhs } => {
                Self::from_if_else(*cond, *lhs, *rhs, count, symtable)
            }
            ast::Node::While { cond, statements } => {
                Self::from_while(*cond, *statements, count, symtable)
            }
            ast::Node::BinaryOp {
                ctype,
                op,
                lhs,
                rhs,
            } => Self::from_binary_op(ctype, op, *lhs, *rhs, count, symtable),
            ast::Node::ConditionalOp {
                ctype,
                op,
                lhs,
                rhs,
            } => Self::from_conditional_op(ctype, op, *lhs, *rhs, count, symtable),
            ast::Node::UnaryOp { ctype, expr } => {
                Self::from_unary_op(ctype, *expr, count, symtable)
            }
            ast::Node::Cast { ctype, expr } => Self::from_cast(ctype, *expr, count, symtable),
            ast::Node::Address { ctype, expr } => Self::from_address(ctype, *expr, count, symtable),
            ast::Node::Dereference { ctype, expr } => {
                Self::from_dereference(ctype, *expr, count, symtable)
            }
            ast::Node::Reference { ctype, expr } => {
                Self::from_reference(ctype, *expr, count, symtable)
            }
            ast::Node::Function {
                ident,
                scope,
                statements,
            } => Self::from_function(ident, scope, *statements, count, symtable),
            ast::Node::Call {
                ctype,
                ident,
                scope,
                arguments,
            } => Self::from_call(ctype, ident, scope, arguments, count, symtable),
            ast::Node::FloatLit { ctype, val } => Self::from_float_lit(ctype, val, count),
            ast::Node::IntLit { ctype, val } => Self::from_int_lit(ctype, val, count),
            ast::Node::Var {
                ctype,
                ident,
                scope,
            } => Self::from_var(ctype, ident, scope, symtable),

            ast::Node::Empty => Ok(Self {
                instructions: VecDeque::new(),
                tmp: None,
            }),
        }
    }

    fn from_assign(
        ctype: CType,
        lhs: ast::Node, // ident
        rhs: ast::Node, // expr
        count: &mut Count,
        symtable: &mut SymTable,
    ) -> Result<Self, Error> {
        let is_addr_assign = matches!(lhs, ast::Node::Address { .. });
        let (mut instructions, lhs) = match lhs {
            ast::Node::Address { expr, .. } => Self::from_ast(*expr, count, symtable)?.split(),
            _ => Self::from_ast(lhs, count, symtable)?.split(),
        };
        let (rhs_instrs, rhs) = Self::from_ast(rhs, count, symtable)?.split();
        instructions.extend(rhs_instrs);
        if is_addr_assign {
            instructions.push_back(Instruction::addr_assign(
                ctype.to_instruction_set(),
                lhs.ok_or_else(|| {
                    Error::ThreeAC(String::from("from_assign: lhs does not have operand"))
                })?,
                rhs.ok_or_else(|| {
                    Error::ThreeAC(String::from("from_assign: rhs does not have operand"))
                })?,
            ));
        } else {
            instructions.push_back(Instruction::assign(
                ctype.to_instruction_set(),
                lhs.ok_or_else(|| {
                    Error::ThreeAC(String::from("from_assign: lhs does not have operand"))
                })?,
                rhs.ok_or_else(|| {
                    Error::ThreeAC(String::from("from_assign: rhs does not have operand"))
                })?,
            ));
        }

        Ok(Self {
            instructions,
            tmp: None,
        })
    }

    fn from_free(
        expr: ast::Node,
        count: &mut Count,
        symtable: &mut SymTable,
    ) -> Result<Self, Error> {
        let (mut instructions, expr) = Self::from_ast(expr, count, symtable)?.split();
        instructions.push_back(Instruction::free(expr.ok_or_else(|| {
            Error::ThreeAC(String::from(
                "from_free: expression does not have an operand",
            ))
        })?));

        Ok(Self {
            instructions,
            tmp: None,
        })
    }

    fn from_malloc(
        ctype: CType,
        expr: ast::Node,
        count: &mut Count,
        symtable: &mut SymTable,
    ) -> Result<Self, Error> {
        let (mut instructions, expr) = Self::from_ast(expr, count, symtable)?.split();
        let tmp = Operand::new_tmp(&ctype, count).ok_or(Error::Type)?;

        instructions.push_back(Instruction::malloc(
            tmp,
            expr.ok_or_else(|| {
                Error::ThreeAC(String::from(
                    "from_malloc: expression does not have an operand",
                ))
            })?,
        ));

        Ok(Self {
            instructions,
            tmp: Some(tmp),
        })
    }

    fn from_read(
        ctype: CType,
        var: ast::Node,
        count: &mut Count,
        symtable: &mut SymTable,
    ) -> Result<Self, Error> {
        let (var_instr, var) = Self::from_ast(var, count, symtable)?.split();
        if !var_instr.is_empty() {
            return Err(Error::ThreeAC(String::from(
                "from_read: var should not have instructions",
            )));
        }

        Ok(Self {
            instructions: VecDeque::from([Instruction::get(
                ctype.to_instruction_set(),
                var.ok_or_else(|| {
                    Error::ThreeAC(String::from("from_read: var does not have operand"))
                })?,
            )]),
            tmp: None,
        })
    }

    fn from_return(
        ctype: CType,
        expr: ast::Node,
        function: i32,
        count: &mut Count,
        symtable: &mut SymTable,
    ) -> Result<Self, Error> {
        let mut instructions: VecDeque<Instruction> = VecDeque::new();
        if !matches!(ctype, CType::Void) {
            let (instr, expr) = Self::from_ast(expr, count, symtable)?.split();
            instructions = instr;
            instructions.push_back(Instruction::save(
                ctype.to_instruction_set(),
                expr.ok_or_else(|| {
                    Error::ThreeAC(String::from(
                        "from_return: expression does not have operand",
                    ))
                })?,
            ));
        }
        instructions.push_back(Instruction::jump(Label::FunctionTail(function)));

        Ok(Self {
            instructions,
            tmp: None,
        })
    }

    fn from_statement_list(
        statements: Vec<ast::Node>,
        count: &mut Count,
        symtable: &mut SymTable,
    ) -> Result<Self, Error> {
        let mut instructions: VecDeque<Instruction> = VecDeque::new();

        for s in statements {
            let (instrs, tmp) = Self::from_ast(s, count, symtable)?.split();
            if tmp.is_some() {
                return Err(Error::ThreeAC(String::from(
                    "from_statement_list: statements should not have operands",
                )));
            }
            instructions.extend(instrs);
        }

        Ok(Self {
            instructions,
            tmp: None,
        })
    }

    fn from_write(
        ctype: CType,
        expr: ast::Node,
        count: &mut Count,
        symtable: &mut SymTable,
    ) -> Result<Self, Error> {
        let (mut instructions, expr) = Self::from_ast(expr, count, symtable)?.split();
        instructions.push_back(Instruction::put(
            ctype.to_instruction_set(),
            expr.ok_or_else(|| {
                Error::ThreeAC(String::from(
                    "from_write: expression does not have an operand",
                ))
            })?,
            ctype == CType::Str,
        ));

        Ok(Self {
            instructions,
            tmp: None,
        })
    }

    fn from_if_else(
        cond: ast::Node,
        lhs: ast::Node,
        rhs: ast::Node,
        count: &mut Count,
        symtable: &mut SymTable,
    ) -> Result<Self, Error> {
        count.label += 1;
        let label = count.label;
        let end = Label::BlockJump(label);

        let (mut instructions, tmp) = Self::from_ast(cond, count, symtable)?.split();
        if tmp.is_some() {
            return Err(Error::ThreeAC(String::from(
                "from_if_else: conditional statements should not have operands",
            )));
        }
        let (linstrs, tmp) = Self::from_ast(lhs, count, symtable)?.split();
        if tmp.is_some() {
            return Err(Error::ThreeAC(String::from(
                "from_if_else: statements should not have operands",
            )));
        }
        let (rinstrs, tmp) = Self::from_ast(rhs, count, symtable)?.split();
        if tmp.is_some() {
            return Err(Error::ThreeAC(String::from(
                "from_if_else: statements should not have operands",
            )));
        }
        instructions.extend(linstrs);
        instructions.push_back(Instruction::jump(end));
        instructions.push_back(Instruction::label(Label::BlockBranch(label)));
        instructions.extend(rinstrs);
        instructions.push_back(Instruction::label(end));

        Ok(Self {
            instructions,
            tmp: None,
        })
    }

    fn from_while(
        cond: ast::Node,
        statements: ast::Node,
        count: &mut Count,
        symtable: &mut SymTable,
    ) -> Result<Self, Error> {
        count.label += 1;
        let label = count.label;
        let head = Label::BlockJump(label);

        let (mut instructions, tmp) = Self::from_ast(cond, count, symtable)?.split();
        if tmp.is_some() {
            return Err(Error::ThreeAC(String::from(
                "from_while: conditional statements should not have operands",
            )));
        }
        let (statements, tmp) = Self::from_ast(statements, count, symtable)?.split();
        if tmp.is_some() {
            return Err(Error::ThreeAC(String::from(
                "from_while: statements should not have operands",
            )));
        }

        instructions.push_front(Instruction::label(head));
        instructions.extend(statements);
        instructions.push_back(Instruction::jump(head));
        instructions.push_back(Instruction::label(Label::BlockBranch(label)));

        Ok(Self {
            instructions,
            tmp: None,
        })
    }

    fn from_binary_op(
        ctype: CType,
        op: BinOp,
        lhs: ast::Node,
        rhs: ast::Node,
        count: &mut Count,
        symtable: &mut SymTable,
    ) -> Result<Self, Error> {
        let (mut instructions, lhs) = Self::from_ast(lhs, count, symtable)?.split();
        let (rhs_instrs, rhs) = Self::from_ast(rhs, count, symtable)?.split();
        instructions.extend(rhs_instrs);

        let tmp = Operand::new_tmp(&ctype, count).ok_or(Error::Type)?;
        instructions.push_back(Instruction {
            variant: match op {
                BinOp::Plus => instruction::Variant::Plus,
                BinOp::Minus => instruction::Variant::Minus,
                BinOp::Times => instruction::Variant::Times,
                BinOp::Divide => instruction::Variant::Divide,
            },
            set: ctype.to_instruction_set(),
            opdt: tmp,
            opm: lhs.ok_or_else(|| {
                Error::ThreeAC(String::from("from_binary_op: lhs does not have an operand"))
            })?,
            opn: rhs.ok_or_else(|| {
                Error::ThreeAC(String::from("from_binary_op: rhs does not have an operand"))
            })?,
        });

        Ok(Self {
            instructions,
            tmp: Some(tmp),
        })
    }

    fn from_conditional_op(
        ctype: CType,
        op: CondOp,
        lhs: ast::Node,
        rhs: ast::Node,
        count: &mut Count,
        symtable: &mut SymTable,
    ) -> Result<Self, Error> {
        let (mut instructions, lhs) = Self::from_ast(lhs, count, symtable)?.split();
        let (rhs_instrs, rhs) = Self::from_ast(rhs, count, symtable)?.split();
        instructions.extend(rhs_instrs);
        instructions.push_back(Instruction {
            variant: match op {
                CondOp::Equal => instruction::Variant::Equal(Label::BlockBranch(count.label)),
                CondOp::NotEqual => instruction::Variant::NotEqual(Label::BlockBranch(count.label)),
                CondOp::Less => instruction::Variant::Less(Label::BlockBranch(count.label)),
                CondOp::LessEqual => {
                    instruction::Variant::LessEqual(Label::BlockBranch(count.label))
                }
                CondOp::Greater => instruction::Variant::Greater(Label::BlockBranch(count.label)),
                CondOp::GreaterEqual => {
                    instruction::Variant::GreaterEqual(Label::BlockBranch(count.label))
                }
            },
            set: ctype.to_instruction_set(),
            opdt: match ctype {
                CType::Float => Operand::new_tmp(&CType::Int, count).ok_or(Error::Type)?,
                _ => Operand::new_null(),
            },
            opm: lhs.ok_or_else(|| {
                Error::ThreeAC(String::from(
                    "from_conditional_op: lhs does not have an operand",
                ))
            })?,
            opn: rhs.ok_or_else(|| {
                Error::ThreeAC(String::from(
                    "from_conditional_op: rhs does not have an operand",
                ))
            })?,
        });

        Ok(Self {
            instructions,
            tmp: None,
        })
    }

    fn from_unary_op(
        ctype: CType,
        expr: ast::Node,
        count: &mut Count,
        symtable: &mut SymTable,
    ) -> Result<Self, Error> {
        let (mut instructions, expr) = Self::from_ast(expr, count, symtable)?.split();
        let tmp = Operand::new_tmp(&ctype, count).ok_or(Error::Type)?;
        instructions.push_back(Instruction::negate(
            ctype.to_instruction_set(),
            tmp,
            expr.ok_or_else(|| {
                Error::ThreeAC(String::from("from_unary_op: expr does not have an operand"))
            })?,
        ));

        Ok(Self {
            instructions,
            tmp: Some(tmp),
        })
    }

    fn from_cast(
        ctype: CType,
        expr: ast::Node,
        count: &mut Count,
        symtable: &mut SymTable,
    ) -> Result<Self, Error> {
        let (mut instructions, expr) = Self::from_ast(expr, count, symtable)?.split();
        let tmp = Operand::new_tmp(&ctype, count).ok_or(Error::Type)?;
        instructions.push_back(Instruction::cast(
            ctype.to_instruction_set(),
            tmp,
            expr.ok_or_else(|| {
                Error::ThreeAC(String::from("from_cast: expr does not have an operand"))
            })?,
        ));

        Ok(Self {
            instructions,
            tmp: Some(tmp),
        })
    }

    fn from_address(
        _ctype: CType,
        expr: ast::Node,
        count: &mut Count,
        symtable: &mut SymTable,
    ) -> Result<Self, Error> {
        let (mut instructions, expr) = Self::from_ast(expr, count, symtable)?.split();
        let tmp = Operand::new_tmp(&CType::Int, count).ok_or(Error::Type)?;
        instructions.push_back(Instruction::address(
            tmp,
            expr.ok_or_else(|| {
                Error::ThreeAC(String::from("from_address: expr does not have an operand"))
            })?,
        ));

        Ok(Self {
            instructions,
            tmp: Some(tmp),
        })
    }

    fn from_dereference(
        ctype: CType,
        expr: ast::Node,
        count: &mut Count,
        symtable: &mut SymTable,
    ) -> Result<Self, Error> {
        let (mut instructions, expr) = Self::from_ast(expr, count, symtable)?.split();
        let tmp = Operand::new_tmp(&ctype, count).ok_or(Error::Type)?;
        instructions.push_back(Instruction::dereference(
            ctype.to_instruction_set(),
            tmp,
            expr.ok_or_else(|| {
                Error::ThreeAC(String::from(
                    "from_dereference: expr does not have an operand",
                ))
            })?,
        ));

        Ok(Self {
            instructions,
            tmp: Some(tmp),
        })
    }

    fn from_reference(
        ctype: CType,
        expr: ast::Node,
        count: &mut Count,
        symtable: &mut SymTable,
    ) -> Result<Self, Error> {
        let (mut instructions, expr) = Self::from_ast(expr, count, symtable)?.split();
        let tmp = Operand::new_tmp(&ctype, count).ok_or(Error::Type)?;
        instructions.push_back(Instruction::reference(
            tmp,
            expr.ok_or_else(|| {
                Error::ThreeAC(String::from(
                    "from_reference: expr does not have an operand",
                ))
            })?,
        ));

        Ok(Self {
            instructions,
            tmp: Some(tmp),
        })
    }

    fn from_function(
        ident: String,
        scope: usize,
        statements: ast::Node,
        count: &mut Count,
        symtable: &mut SymTable,
    ) -> Result<Self, Error> {
        count.reset();

        let symbol = symtable.get_symbol_in_scope(&ident, scope)?;
        symtable.switch_scope(symbol.scope())?;
        let (_, local_offset) = symtable.get_addr_offset_val();

        let (mut instructions, tmp) = Self::from_ast(statements, count, symtable)?.split();
        if tmp.is_some() {
            return Err(Error::ThreeAC(String::from(
                "from_statement_list: statements should not have operands",
            )));
        }

        if local_offset < 0 {
            instructions.push_front(Instruction::alloc(local_offset));
        }
        instructions.push_front(Instruction::label(Label::FunctionHead(symbol.address())));
        instructions.push_back(Instruction::label(Label::FunctionTail(symbol.address())));
        instructions.push_back(Instruction::ret());

        symtable.pop_scope()?;
        Ok(Self {
            instructions,
            tmp: None,
        })
    }

    fn from_call(
        ctype: CType,
        ident: String,
        scope: usize,
        arguments: Vec<ast::Node>,
        count: &mut Count,
        symtable: &mut SymTable,
    ) -> Result<Self, Error> {
        let mut args = Vec::new();
        let mut instructions = VecDeque::new();

        for a in arguments {
            let (instrs, a) = Self::from_ast(a, count, symtable)?.split();
            instructions.extend(instrs);
            args.push(a.ok_or_else(|| {
                Error::ThreeAC(String::from("from_call: expression does not have operand"))
            })?);
        }

        let tmp = Operand::new_tmp(&ctype, count);
        instructions.push_back(Instruction::call(
            Label::FunctionHead(symtable.get_symbol_in_scope(&ident, scope)?.address()),
            args,
            ctype.to_instruction_set(),
            match &tmp {
                Some(o) => *o,
                None => Operand::new_null(),
            },
        ));

        Ok(Self { instructions, tmp })
    }

    fn from_float_lit(ctype: CType, val: f32, count: &mut Count) -> Result<Self, Error> {
        let tmp = Operand::new_tmp(&ctype, count).ok_or(Error::Type)?;
        Ok(Self {
            instructions: VecDeque::from([Instruction::load(
                ctype.to_instruction_set(),
                tmp,
                val.to_string(),
            )]),
            tmp: Some(tmp),
        })
    }

    fn from_int_lit(ctype: CType, val: i32, count: &mut Count) -> Result<Self, Error> {
        let tmp = Operand::new_tmp(&ctype, count).ok_or(Error::Type)?;
        Ok(Self {
            instructions: VecDeque::from([Instruction::load(
                ctype.to_instruction_set(),
                tmp,
                val.to_string(),
            )]),
            tmp: Some(tmp),
        })
    }

    fn from_var(
        ctype: CType,
        ident: String,
        scope: usize,
        symtable: &SymTable,
    ) -> Result<Self, Error> {
        let symbol = symtable.get_symbol_in_scope(&ident, scope)?;

        Ok(Self {
            instructions: VecDeque::new(),
            tmp: Some(Operand::from_symbol(&ctype, symbol)?),
        })
    }

    fn split(self) -> (VecDeque<Instruction>, Option<Operand>) {
        (self.instructions, self.tmp)
    }

    pub fn add_headers(mut self, func: i32, strs: String) -> Self {
        self.instructions
            .push_front(Instruction::header_text(Label::FunctionHead(func)));
        self.instructions
            .push_back(Instruction::header_strings(strs));

        self
    }
}

impl fmt::Display for Instructions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut format = String::new();
        for i in &self.instructions {
            format = format!("{format}{i}");
        }

        write!(f, "{format}")
    }
}

impl CType {
    fn to_instruction_set(&self) -> instruction::Set {
        match self {
            Self::Float => instruction::Set::F,
            _ => instruction::Set::T,
        }
    }
}

impl Operand {
    fn new_tmp(ctype: &CType, count: &mut Count) -> Option<Self> {
        match ctype {
            CType::Void => None,
            CType::Float => {
                count.float += 1;
                Some(Operand {
                    variant: operand::Variant::TempFloat(count.float),
                    otype: operand::Type::F,
                })
            }
            _ => {
                count.regular += 1;
                Some(Operand {
                    variant: operand::Variant::Temp(count.regular),
                    otype: operand::Type::T,
                })
            }
        }
    }
}
