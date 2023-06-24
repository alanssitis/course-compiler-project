use pest::iterators::Pairs;
use pest::Parser;
use std::fs;

use crate::ast;
use crate::error::Error;
use crate::symtable::{CType, SymTable, SymbolType};

const GLOBAL_BASE: i32 = 0x20000000;
const STRING_BASE: i32 = 0x10000000;

#[derive(Parser)]
#[grammar = "micro_c.pest"]
pub struct MicroC;

pub fn parse_file(path: &String) -> Result<(ast::Node, SymTable), Error> {
    let unparsed = fs::read_to_string(path)?;
    let parse_result = MicroC::parse(Rule::program, &unparsed)?;
    parse_program(parse_result)
}

// Build AST and SymTable from the declaration and function trees.
pub fn parse_program(pairs: Pairs<Rule>) -> Result<(ast::Node, SymTable), Error> {
    let mut statements: Vec<ast::Node> = Vec::new();
    let mut symtable = SymTable::new(GLOBAL_BASE, STRING_BASE);

    for pair in pairs {
        match pair.as_rule() {
            // Declarations
            Rule::func_decl => {
                let mut subpairs = pair.into_inner();
                let ctype =
                    CType::from_base_type(subpairs.next().ok_or(Error::PairsNext)?.into_inner());
                let name = subpairs.next().ok_or(Error::PairsNext)?.as_str().to_owned();
                let arguments: Vec<CType> = subpairs
                    .next()
                    .ok_or(Error::PairsNext)?
                    .into_inner()
                    .map(|pair| match pair.into_inner().peek() {
                        Some(p) => CType::from_base_type(p.into_inner()),
                        None => unreachable!("expected base_type, got nothing"),
                    })
                    .collect();
                symtable.add_function(ctype, name, arguments)?;
            }
            Rule::var_decl => {
                let mut subpairs = pair.into_inner();
                let ctype =
                    CType::from_base_type(subpairs.next().ok_or(Error::PairsNext)?.into_inner());
                symtable.add_symbol(
                    ctype,
                    subpairs.next().ok_or(Error::PairsNext)?.as_str().to_owned(),
                    SymbolType::Global,
                )?;
            }
            Rule::str_decl => {
                let mut subpairs = pair.into_inner();
                symtable.add_symbol(
                    CType::Str,
                    subpairs.next().ok_or(Error::PairsNext)?.as_str().to_owned(),
                    SymbolType::Str(subpairs.next().ok_or(Error::PairsNext)?.as_str().to_owned()),
                )?;
            }
            Rule::function => statements.push(ast::construct::from_function(pair, &mut symtable)?),
            r => {
                return Err(Error::Other(format!(
                    "parse_program: expected decls and function, found {:?}",
                    r
                )))
            }
        }
    }

    Ok((ast::Node::StatementList { statements }, symtable))
}
