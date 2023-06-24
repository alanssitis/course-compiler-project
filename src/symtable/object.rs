use super::ctype::CType;
use super::entry::Entry;
use super::entry::SymbolType;
use super::global;
use super::local;
use super::scope::Scope;

use crate::error::Error;

#[derive(Debug)]
pub struct SymTable {
    scopes: Vec<Scope>,
    curr: usize,
}

const ARG_OFFSET: i32 = 8;
const VAR_OFFSET: i32 = 0;

impl SymTable {
    pub fn new(global_base: i32, string_base: i32) -> SymTable {
        let scopes = vec![Scope::Global(global::Scope::new(global_base, string_base))];

        SymTable { scopes, curr: 0 }
    }

    pub fn switch_scope(&mut self, scope: usize) -> Result<(), Error> {
        if scope >= self.scopes.len() {
            return Err(Error::SymTable(format!(
                "switch_cope: {scope} is outside of range"
            )));
        }
        self.curr = scope;
        Ok(())
    }

    pub fn push_scope(
        &mut self,
        function: i32,
        name: &String,
        ret_val: CType,
    ) -> Result<(), Error> {
        let parent = self.curr;
        self.curr = self.scopes.len();
        self.scopes.push(Scope::Local(local::Scope::new(
            function, parent, ARG_OFFSET, VAR_OFFSET, ret_val,
        )));
        self.scopes[parent].add_subscope(self.curr)?;
        self.scopes[parent].set_function_scope(name, self.curr)
    }

    pub fn pop_scope(&mut self) -> Result<(), Error> {
        self.curr = match self.scopes[self.curr].get_parent() {
            Some(val) => val,
            None => {
                return Err(Error::SymTable(String::from(
                    "pop_scope: cannot pop from global scope",
                )))
            }
        };

        Ok(())
    }

    pub fn add_function(
        &mut self,
        ctype: CType,
        name: String,
        arguments: Vec<CType>,
    ) -> Result<(), Error> {
        self.scopes[self.curr].add_function(ctype, name, arguments)
    }

    pub fn add_symbol(
        &mut self,
        ctype: CType,
        name: String,
        symtype: SymbolType,
    ) -> Result<(), Error> {
        self.scopes[self.curr].add_symbol(ctype, name, symtype)
    }

    pub fn contains_symbol(&self, name: &String) -> bool {
        let mut ret = self.scopes[self.curr].contains_symbol(name);
        if !ret {
            let mut curr = self.curr;
            while let Some(parent) = self.scopes[curr].get_parent() {
                ret = self.scopes[parent].contains_symbol(name);
                curr = parent;
            }
        }
        ret
    }

    pub fn get_function(&self) -> Result<i32, Error> {
        self.scopes[self.curr].get_function()
    }

    pub fn get_scope(&self, name: &String) -> Result<usize, Error> {
        if self.contains_symbol(name) {
            let mut curr = self.curr;
            while let Some(parent) = self.scopes[curr].get_parent() {
                if self.scopes[curr].contains_symbol(name) {
                    return Ok(curr);
                }
                curr = parent;
            }
            if self.scopes[curr].contains_symbol(name) {
                return Ok(curr);
            }
        }
        Err(Error::SymTable(format!(
            "get_scope: symtable does not contain {name}"
        )))
    }

    pub fn get_symbol_in_scope(&self, name: &String, scope: usize) -> Result<Entry, Error> {
        match self.scopes[scope].get_symbol(name) {
            Some(e) => Ok(e),
            None => Err(Error::SymTable(format!(
                "get_symbol_in_scope: failed to get symbol {name} in scope {scope}"
            ))),
        }
    }

    pub fn get_symbol(&self, name: &String) -> Result<Entry, Error> {
        if self.contains_symbol(name) {
            let mut curr = self.curr;
            while let Some(parent) = self.scopes[curr].get_parent() {
                if let Some(sym) = self.scopes[curr].get_symbol(name) {
                    return Ok(sym);
                }
                curr = parent;
            }
            return self.get_symbol_in_scope(name, curr);
        }
        Err(Error::SymTable(format!(
            "get_scope: symtable does not contain {name}"
        )))
    }

    pub fn get_scope_ctype(&self) -> Result<CType, Error> {
        self.scopes[self.curr].get_scope_ctype()
    }

    pub fn strings_in_asm(&self) -> String {
        let mut ret = String::new();
        for scope in &self.scopes {
            match scope {
                Scope::Global(scope) => ret.push_str(&scope.strings_in_asm()),
                Scope::Local(_) => {}
            }
        }
        ret
    }

    pub fn get_addr_offset_val(&self) -> (i32, i32) {
        self.scopes[self.curr].get_addr_offset_val()
    }
}
