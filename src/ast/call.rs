use pest::iterators::Pair;

use super::climbers;
use super::Node;

use crate::error::Error;
use crate::parser::Rule;
use crate::symtable::{CType, SymTable};

pub fn from_pair_to_node(pair: Pair<Rule>, symtable: &SymTable) -> Result<Node, Error> {
    match pair.as_rule() {
        Rule::func_call => from_func_call(pair, symtable),
        Rule::read_stmt => {
            let ident = pair
                .into_inner()
                .next()
                .ok_or(Error::PairsNext)?
                .as_str()
                .to_owned();
            let ctype = symtable.get_symbol(&ident)?.ctype();
            Ok(Node::Read {
                ctype: ctype.clone(),
                var: Box::new(Node::Var {
                    ctype,
                    scope: symtable.get_scope(&ident)?,
                    ident,
                }),
            })
        }
        Rule::print_stmt => {
            let expr = climbers::from_expr(
                pair.into_inner()
                    .next()
                    .ok_or(Error::PairsNext)?
                    .into_inner(),
                symtable,
            )?;
            Ok(Node::Write {
                ctype: expr.ctype(),
                expr: Box::new(expr),
            })
        }
        Rule::malloc_stmt => {
            let expr = climbers::from_expr(
                pair.into_inner()
                    .next()
                    .ok_or(Error::PairsNext)?
                    .into_inner(),
                symtable,
            )?;
            Ok(Node::Malloc {
                ctype: CType::Ptr(Box::new(CType::Void)),
                expr: Box::new(expr),
            })
        }
        Rule::free_stmt => {
            let expr = climbers::from_expr(
                pair.into_inner()
                    .next()
                    .ok_or(Error::PairsNext)?
                    .into_inner(),
                symtable,
            )?;
            if !matches!(expr.ctype(), CType::Ptr(_)) {
                return Err(Error::Type);
            }
            Ok(Node::Free {
                expr: Box::new(expr),
            })
        }
        _ => unreachable!("from_pair_to_node: expected call, got other"),
    }
}

fn from_func_call(pair: Pair<Rule>, symtable: &SymTable) -> Result<Node, Error> {
    let mut sp = pair.into_inner();
    let ident = sp.next().ok_or(Error::PairsNext)?.as_str().to_owned();
    let symbol = symtable.get_symbol(&ident)?;
    let mut arguments = Vec::new();
    for (p, ct) in sp.zip(symbol.arguments()?) {
        arguments.push(climbers::from_expr(p.into_inner(), symtable)?.set_ctype(&ct)?);
    }
    if arguments.len() != symbol.arguments()?.len() {
        return Err(Error::Type);
    }
    Ok(Node::Call {
        ctype: symbol.ctype(),
        scope: symtable.get_scope(&ident)?,
        ident,
        arguments,
    })
}
