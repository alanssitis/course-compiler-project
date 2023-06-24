use super::ctype::CType;
use super::entry::Entry;
use super::entry::SymbolType;
use super::global;
use super::local;

use crate::error::Error;

#[derive(Debug)]
pub enum Scope {
    Global(global::Scope),
    Local(local::Scope),
}

impl Scope {
    pub fn add_subscope(&mut self, child: usize) -> Result<(), Error> {
        match self {
            Scope::Global(scope) => {
                scope.add_subscope(child);
                Ok(())
            }
            Scope::Local(_) => Err(Error::SymTable(String::from(
                "add_subscope: local scope cannot add subscopes",
            ))),
        }
    }

    pub fn add_function(
        &mut self,
        ctype: CType,
        name: String,
        arguments: Vec<CType>,
    ) -> Result<(), Error> {
        match self {
            Scope::Global(scope) => {
                scope.add_function(ctype, name, arguments);
                Ok(())
            }
            Scope::Local(_) => Err(Error::SymTable(String::from(
                "add_function: local scope cannot add functions",
            ))),
        }
    }

    pub fn set_function_scope(&mut self, name: &String, scope: usize) -> Result<(), Error> {
        match self {
            Scope::Global(s) => Ok(s.set_function_scope(name, scope)?),
            Scope::Local(_) => Err(Error::SymTable(String::from(
                "set_function_scope: local scope should not have functions",
            ))),
        }
    }

    pub fn add_symbol(
        &mut self,
        ctype: CType,
        name: String,
        symtype: SymbolType,
    ) -> Result<(), Error> {
        match self {
            Scope::Global(scope) => scope.add_symbol(ctype, name, symtype)?,
            Scope::Local(scope) => scope.add_symbol(ctype, name, symtype)?,
        }

        Ok(())
    }

    pub fn contains_symbol(&self, name: &String) -> bool {
        match self {
            Scope::Global(scope) => scope.contains_symbol(name),
            Scope::Local(scope) => scope.contains_symbol(name),
        }
    }

    pub fn get_parent(&self) -> Option<usize> {
        match self {
            Scope::Global(scope) => scope.get_parent(),
            Scope::Local(scope) => scope.get_parent(),
        }
    }

    pub fn get_symbol(&self, name: &String) -> Option<Entry> {
        match self {
            Scope::Global(scope) => scope.get_symbol(name),
            Scope::Local(scope) => scope.get_symbol(name),
        }
    }

    pub fn get_addr_offset_val(&self) -> (i32, i32) {
        match self {
            Scope::Global(scope) => scope.get_addr_val(),
            Scope::Local(scope) => scope.get_offset_val(),
        }
    }

    pub fn get_function(&self) -> Result<i32, Error> {
        match self {
            Scope::Global(_) => Err(Error::SymTable(String::from(
                "get_function: global scope is not a function",
            ))),
            Scope::Local(scope) => Ok(scope.get_function()),
        }
    }

    pub fn get_scope_ctype(&self) -> Result<CType, Error> {
        match self {
            Scope::Global(_) => Err(Error::SymTable(String::from(
                "get_scope_ctype: global scope is not under a function",
            ))),
            Scope::Local(scope) => Ok(scope.get_scope_ctype()),
        }
    }
}
