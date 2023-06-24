use std::collections::HashMap;

use super::ctype::CType;
use super::entry::Entry;
use super::entry::SymbolType;

use crate::error::Error;

#[derive(Debug)]
pub struct Scope {
    table: HashMap<String, Entry>,
    children: Vec<usize>,
    global_base: i32,
    string_base: i32,
    function_base: i32,
}

impl Scope {
    pub fn new(global_base: i32, string_base: i32) -> Scope {
        Scope {
            table: HashMap::new(),
            children: Vec::new(),
            global_base,
            string_base,
            function_base: 0,
        }
    }

    pub fn add_subscope(&mut self, child: usize) {
        self.children.push(child);
    }

    pub fn add_function(&mut self, ctype: CType, name: String, arguments: Vec<CType>) {
        self.table.insert(
            name,
            Entry::Function {
                ctype,
                address: self.function_base,
                scope: 0,
                arguments,
            },
        );
        self.function_base += 1;
    }

    pub fn set_function_scope(&mut self, name: &String, scope: usize) -> Result<(), Error> {
        match self.table.get(name) {
            Some(e) => self.table.insert(name.clone(), e.clone().set_scope(scope)),
            None => {
                return Err(Error::SymTable(String::from(
                    "set_function_scope: function doesn't exist",
                )))
            }
        };
        Ok(())
    }

    pub fn add_symbol(
        &mut self,
        ctype: CType,
        name: String,
        symtype: SymbolType,
    ) -> Result<(), Error> {
        self.table.insert(
            name,
            Entry::Symbol {
                ctype,
                address: match symtype {
                    SymbolType::Global => {
                        let addr = self.global_base;
                        self.global_base += 4;
                        addr
                    }
                    SymbolType::Str(_) => {
                        let addr = self.string_base;
                        self.string_base += 4;
                        addr
                    }
                    s => {
                        return Err(Error::SymTable(format!(
                            "add_symbol: expected global or string, got {s:?}"
                        )))
                    }
                },
                symtype,
            },
        );
        Ok(())
    }

    pub fn contains_symbol(&self, name: &String) -> bool {
        self.table.contains_key(name)
    }

    pub fn get_parent(&self) -> Option<usize> {
        None
    }

    pub fn get_symbol(&self, name: &String) -> Option<Entry> {
        Some(self.table.get(name)?.clone())
    }

    pub fn strings_in_asm(&self) -> String {
        let mut string = String::new();

        for entry in self.table.values() {
            if let Entry::Symbol {
                address,
                symtype: SymbolType::Str(value),
                ..
            } = entry
            {
                string.push_str(&format!("0x{:08x} {}\n", address, value));
            }
        }

        string
    }

    pub fn get_addr_val(&self) -> (i32, i32) {
        (self.global_base, self.string_base)
    }
}
