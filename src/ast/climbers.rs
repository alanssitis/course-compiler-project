use pest::iterators::{Pair, Pairs};
use pest::pratt_parser::Assoc::Left;
use pest::pratt_parser::{Op, PrattParser};

use super::call;
use super::{BinOp, CondOp, Node};

use crate::error::Error;
use crate::parser::Rule;
use crate::symtable::CType;
use crate::symtable::SymTable;

// Expr prec_climber
lazy_static::lazy_static! {
    static ref EXPR_CLIMBER: PrattParser<Rule> = PrattParser::new()
        .op(Op::infix(Rule::plus, Left) | Op::infix(Rule::minus, Left))
        .op(Op::infix(Rule::times, Left) | Op::infix(Rule::divide, Left))
        .op(Op::prefix(Rule::neg) | Op::prefix(Rule::base_type)
            | Op::prefix(Rule::dereference) | Op::prefix(Rule::reference))
        .op(Op::postfix(Rule::array_expr));

    static ref COND_CLIMBER: PrattParser<Rule> = PrattParser::new()
        .op(Op::infix(Rule::equal, Left) | Op::infix(Rule::not_equal, Left))
        .op(Op::infix(Rule::less, Left) | Op::infix(Rule::less_equal, Left)
            | Op::infix(Rule::greater, Left) | Op::infix(Rule::greater_equal, Left));

    static ref LVAL_CLIMBER: PrattParser<Rule> = PrattParser::new()
        .op(Op::infix(Rule::plus, Left) | Op::infix(Rule::minus, Left))
        .op(Op::infix(Rule::times, Left) | Op::infix(Rule::divide, Left))
        .op(Op::prefix(Rule::neg) | Op::prefix(Rule::base_type)
            | Op::prefix(Rule::address))
        .op(Op::postfix(Rule::array_expr));
}

// Build AST for an expr.
pub fn from_expr(pairs: Pairs<Rule>, symtable: &SymTable) -> Result<Node, Error> {
    EXPR_CLIMBER
        .map_primary(|p| match p.as_rule() {
            Rule::expr => from_expr(p.into_inner(), symtable),
            Rule::call => {
                call::from_pair_to_node(p.into_inner().peek().ok_or(Error::PairsNext)?, symtable)
            }
            Rule::ident => {
                let ident = p.as_str().to_owned();
                Ok(Node::Var {
                    ctype: symtable.get_symbol(&ident)?.ctype(),
                    scope: symtable.get_scope(&ident)?,
                    ident,
                })
            }
            Rule::int_lit => Ok(Node::IntLit {
                ctype: CType::Int,
                val: p.as_str().parse::<i32>().unwrap(),
            }),
            Rule::float_lit => Ok(Node::FloatLit {
                ctype: CType::Float,
                val: p.as_str().parse::<f32>().unwrap(),
            }),
            _ => unreachable!("from_expr: expected atom, found something else"),
        })
        .map_prefix(|op, rhs| {
            let rhs = rhs?;
            match op.as_rule() {
                Rule::neg => Ok(Node::UnaryOp {
                    ctype: rhs.ctype(),
                    expr: Box::new(rhs),
                }),
                Rule::base_type => Ok(Node::Cast {
                    ctype: match CType::from_base_type(op.into_inner()) {
                        CType::Int => CType::Int,
                        CType::Float => CType::Float,
                        _ => return Err(Error::Type),
                    },
                    expr: Box::new(rhs),
                }),
                Rule::dereference => Ok(Node::Dereference {
                    ctype: rhs.ctype().dereference()?,
                    expr: Box::new(rhs),
                }),
                Rule::reference => match rhs {
                    Node::Dereference { expr, .. } => Ok(*expr),
                    _ => Ok(Node::Reference {
                        ctype: CType::Ptr(Box::new(rhs.ctype())),
                        expr: Box::new(rhs),
                    }),
                },
                _ => unreachable!(
                    "from_expr: expected neg, cast, dereference or reference, found other"
                ),
            }
        })
        .map_postfix(|lhs, op| match op.as_rule() {
            Rule::array_expr => {
                let arr_expr = resolve_array_expr(
                    lhs?,
                    op.into_inner().peek().ok_or(Error::PairsNext)?,
                    symtable,
                )?;
                Ok(Node::Dereference {
                    ctype: arr_expr.ctype().dereference()?,
                    expr: Box::new(arr_expr),
                })
            }
            _ => unreachable!("from_expr: expected ident, found other"),
        })
        .map_infix(|lhs, op, rhs| {
            let op = match op.as_rule() {
                Rule::plus => BinOp::Plus,
                Rule::minus => BinOp::Minus,
                Rule::times => BinOp::Times,
                Rule::divide => BinOp::Divide,
                _ => unreachable!("from_expr: expected bin_op, found other"),
            };

            let mut lhs = lhs?;
            let mut rhs = rhs?;

            let lctype = lhs.ctype();
            let rctype = rhs.ctype();
            if lctype != rctype {
                if lctype == CType::Int && rctype == CType::Float {
                    lhs = lhs.cast(&CType::Float);
                } else if rctype == CType::Int && lctype == CType::Float {
                    rhs = rhs.cast(&CType::Float);
                } else {
                    return Err(Error::Type);
                }
            }
            Ok(Node::BinaryOp {
                ctype: lhs.ctype(),
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            })
        })
        .parse(pairs)
}

// Build AST for a cond.
pub fn from_cond(pairs: Pairs<Rule>, symtable: &SymTable) -> Result<Node, Error> {
    COND_CLIMBER
        .map_primary(|p| match p.as_rule() {
            Rule::expr => from_expr(p.into_inner(), symtable),
            _ => unreachable!("from_cond: expected expr, found other"),
        })
        .map_infix(|lhs, op, rhs| {
            let op = match op.as_rule() {
                Rule::equal => CondOp::Equal,
                Rule::not_equal => CondOp::NotEqual,
                Rule::less => CondOp::Less,
                Rule::less_equal => CondOp::LessEqual,
                Rule::greater => CondOp::Greater,
                Rule::greater_equal => CondOp::GreaterEqual,
                _ => unreachable!("from_cond: expected cmp_op, found other"),
            };
            let mut lhs = lhs?;
            let mut rhs = rhs?;
            if lhs.ctype() != rhs.ctype() {
                rhs = rhs.set_ctype(&CType::Float)?;
                lhs = lhs.set_ctype(&CType::Float)?;
            }
            Ok(Node::ConditionalOp {
                ctype: lhs.ctype(),
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            })
        })
        .parse(pairs)
}

pub fn from_lval(pairs: Pairs<Rule>, symtable: &SymTable) -> Result<Node, Error> {
    LVAL_CLIMBER
        .map_primary(|p| match p.as_rule() {
            Rule::lval => from_lval(p.into_inner(), symtable),
            Rule::ident => {
                let ident = p.as_str().to_owned();
                Ok(Node::Var {
                    ctype: symtable.get_symbol(&ident)?.ctype(),
                    scope: symtable.get_scope(&ident)?,
                    ident,
                })
            }
            Rule::int_lit => Ok(Node::IntLit {
                ctype: CType::Int,
                val: p.as_str().parse::<i32>().unwrap(),
            }),
            _ => unreachable!("from_lval: expected unit, found something else"),
        })
        .map_prefix(|op, rhs| {
            let rhs = rhs?;
            match op.as_rule() {
                Rule::neg => Ok(Node::UnaryOp {
                    ctype: rhs.ctype(),
                    expr: Box::new(rhs),
                }),
                Rule::base_type => Ok(Node::Cast {
                    ctype: match CType::from_base_type(op.into_inner()) {
                        CType::Int => CType::Int,
                        CType::Float => CType::Float,
                        _ => return Err(Error::Type),
                    },
                    expr: Box::new(rhs),
                }),
                Rule::address => Ok(Node::Address {
                    ctype: rhs.ctype(),
                    expr: Box::new(rhs),
                }),
                _ => unreachable!("from_lval: expected neg, cast or address, found other"),
            }
        })
        .map_postfix(|lhs, op| match op.as_rule() {
            Rule::array_expr => {
                let arr_expr = resolve_array_expr(
                    lhs?,
                    op.into_inner().peek().ok_or(Error::PairsNext)?,
                    symtable,
                )?;
                Ok(Node::Address {
                    ctype: arr_expr.ctype(),
                    expr: Box::new(arr_expr),
                })
            }
            _ => unreachable!("from_lval: expected array_expr, found other"),
        })
        .map_infix(|lhs, op, rhs| {
            let op = match op.as_rule() {
                Rule::plus => BinOp::Plus,
                Rule::minus => BinOp::Minus,
                Rule::times => BinOp::Times,
                Rule::divide => BinOp::Divide,
                _ => unreachable!("from_lval: expected bin_op, found other"),
            };

            let mut lhs = lhs?;
            let mut rhs = rhs?;

            let lctype = lhs.ctype();
            let rctype = rhs.ctype();
            if lctype != rctype {
                if lctype == CType::Int && matches!(rctype, CType::Ptr(_)) {
                    lhs = lhs.set_ctype(&rctype)?;
                } else if rctype == CType::Int && matches!(lctype, CType::Ptr(_)) {
                    rhs = rhs.set_ctype(&lctype)?;
                } else {
                    return Err(Error::Type);
                }
            }
            Ok(Node::BinaryOp {
                ctype: lhs.ctype(),
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            })
        })
        .parse(pairs)
}

fn resolve_array_expr(lhs: Node, expr: Pair<Rule>, symtable: &SymTable) -> Result<Node, Error> {
    let expr = from_expr(expr.into_inner(), symtable)?;
    Ok(Node::BinaryOp {
        ctype: lhs.ctype(),
        op: BinOp::Plus,
        rhs: Box::new(Node::BinaryOp {
            ctype: expr.ctype(),
            op: BinOp::Times,
            lhs: Box::new(expr),
            rhs: Box::new(
                Node::IntLit {
                    ctype: CType::Int,
                    val: 4,
                }
                .set_ctype(&lhs.ctype())?,
            ),
        }),
        lhs: Box::new(lhs),
    })
}
