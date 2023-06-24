use pest::iterators::Pair;

use super::Node;
use super::{call, climbers};

use crate::error::Error;
use crate::parser::Rule;
use crate::symtable::CType;
use crate::symtable::SymTable;
use crate::symtable::SymbolType;

// Build AST for functions.
pub fn from_function(pair: Pair<Rule>, symtable: &mut SymTable) -> Result<Node, Error> {
    let mut subpairs = pair.into_inner();
    let ret_type = CType::from_base_type(subpairs.next().ok_or(Error::PairsNext)?.into_inner());
    let name = subpairs.next().ok_or(Error::PairsNext)?.as_str().to_owned();
    let params = extract_params(subpairs.next().ok_or(Error::PairsNext)?)?;

    if !symtable.contains_symbol(&name) {
        symtable.add_function(
            ret_type.clone(),
            name.clone(),
            params.iter().cloned().map(|(ctype, _)| ctype).collect(),
        )?;
    }
    if ret_type != symtable.get_symbol(&name)?.ctype() {
        return Err(Error::Type);
    }

    symtable.push_scope(symtable.get_symbol(&name)?.address(), &name, ret_type)?;
    for (ctype, ident) in params.into_iter() {
        symtable.add_symbol(ctype, ident, SymbolType::Argument)?;
    }

    let mut statements = Node::Empty;
    for p in subpairs {
        match p.as_rule() {
            Rule::var_decl => {
                let mut subpairs = p.into_inner();
                let ctype =
                    CType::from_base_type(subpairs.next().ok_or(Error::PairsNext)?.into_inner());
                symtable.add_symbol(
                    ctype,
                    subpairs.next().ok_or(Error::PairsNext)?.as_str().to_owned(),
                    SymbolType::Local,
                )?;
            }
            Rule::statements => statements = from_statements(p, symtable)?,
            r => {
                return Err(Error::Other(format!(
                    "from_function: expected function content, got {r:?}"
                )))
            }
        };
    }
    symtable.pop_scope()?;

    Ok(Node::Function {
        scope: symtable.get_scope(&name)?,
        ident: name,
        statements: Box::new(statements),
    })
}

fn extract_params(pair: Pair<Rule>) -> Result<Vec<(CType, String)>, Error> {
    if pair.as_rule() != Rule::params {
        return Err(Error::Other(String::from(
            "extract_params: pair is not params",
        )));
    }

    let mut ret = Vec::new();
    for p in pair.into_inner() {
        let mut subpairs = p.into_inner();
        let ctype = CType::from_base_type(subpairs.next().ok_or(Error::PairsNext)?.into_inner());
        ret.push((
            ctype,
            subpairs.next().ok_or(Error::PairsNext)?.as_str().to_owned(),
        ));
    }
    Ok(ret)
}

// Build AST for statements.
fn from_statements(pair: Pair<Rule>, symtable: &SymTable) -> Result<Node, Error> {
    let mut statements: Vec<Node> = Vec::new();

    for p in pair.into_inner() {
        statements.push(from_statement(p, symtable)?);
    }

    if statements.is_empty() {
        return Err(Error::Other(String::from("from_statements: no statements")));
    }
    Ok(Node::StatementList { statements })
}

// Build AST for a statement.
fn from_statement(pair: Pair<Rule>, symtable: &SymTable) -> Result<Node, Error> {
    match pair.as_rule() {
        // base statements
        Rule::call => call::from_pair_to_node(
            pair.into_inner()
                .peek()
                .ok_or_else(|| Error::Other(String::from("from_statement: call is empty")))?,
            symtable,
        ),
        Rule::assign_stmt => {
            let mut pairs = pair.into_inner();
            let lhs =
                climbers::from_lval(pairs.next().ok_or(Error::PairsNext)?.into_inner(), symtable)?;
            let ctype = match &lhs {
                Node::Address { .. } => lhs.strip_ctype()?,
                _ => lhs.ctype(),
            };
            let mut rhs =
                climbers::from_expr(pairs.next().ok_or(Error::PairsNext)?.into_inner(), symtable)?;
            if rhs.ctype() != ctype {
                if matches!(ctype, CType::Int | CType::Float) {
                    rhs = rhs.cast(&ctype);
                } else {
                    rhs = rhs.set_ctype(&ctype)?;
                }
            }

            Ok(Node::Assign {
                ctype,
                rhs: Box::new(rhs),
                lhs: Box::new(lhs),
            })
        }
        Rule::return_stmt => {
            let ctype = symtable.get_scope_ctype()?;
            match pair.into_inner().peek() {
                Some(p) => {
                    let expr = climbers::from_expr(p.into_inner(), symtable)?.set_ctype(&ctype)?;
                    Ok(Node::Return {
                        ctype,
                        function: symtable.get_function()?,
                        expr: Box::new(expr),
                    })
                }
                None => {
                    if !matches!(ctype, CType::Void) {
                        return Err(Error::Type);
                    }
                    Ok(Node::Return {
                        ctype,
                        function: symtable.get_function()?,
                        expr: Box::new(Node::Empty),
                    })
                }
            }
        }

        // block statements
        Rule::if_stmt => {
            let mut pairs = pair.into_inner();
            let cond =
                climbers::from_cond(pairs.next().ok_or(Error::PairsNext)?.into_inner(), symtable)?;
            let lhs = from_statements(pairs.next().ok_or(Error::PairsNext)?, symtable)?;
            let rhs = match pairs.next() {
                Some(pair) => from_statements(pair, symtable)?,
                None => Node::Empty,
            };
            Ok(Node::IfElse {
                cond: Box::new(cond),
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            })
        }
        Rule::while_stmt => {
            let mut pairs = pair.into_inner();
            let cond =
                climbers::from_cond(pairs.next().ok_or(Error::PairsNext)?.into_inner(), symtable)?;
            Ok(Node::While {
                cond: Box::new(cond),
                statements: Box::new(from_statements(
                    pairs.next().ok_or(Error::PairsNext)?,
                    symtable,
                )?),
            })
        }

        // Unexpected
        r => Err(Error::Other(format!(
            "from_statement: expected statement, found {r:?}",
        ))),
    }
}
