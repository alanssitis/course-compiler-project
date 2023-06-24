use std::collections::HashMap;

use super::ctype::CType;
use super::entry::Entry;
use super::entry::SymbolType;

use crate::error::Error;

#[derive(Debug)]
pub struct Scope {
    table: HashMap<String, Entry>,
    function: i32,
    parent: usize,
    arg_offset: i32,
    var_offset: i32,
    ret_type: CType,
}

impl Scope {
    pub fn new(
        function: i32,
        parent: usize,
        arg_offset: i32,
        var_offset: i32,
        ret_type: CType,
    ) -> Scope {
        Scope {
            table: HashMap::new(),
            function,
            parent,
            arg_offset,
            var_offset,
            ret_type,
        }
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
                    SymbolType::Argument => {
                        self.arg_offset += 4;
                        self.arg_offset
                    }
                    SymbolType::Local => {
                        self.var_offset -= 4;
                        self.var_offset
                    }
                    s => {
                        return Err(Error::SymTable(format!(
                            "expected arguments or local, got {s:?}"
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

    pub fn get_function(&self) -> i32 {
        self.function
    }

    pub fn get_parent(&self) -> Option<usize> {
        Some(self.parent)
    }

    pub fn get_symbol(&self, name: &String) -> Option<Entry> {
        Some(self.table.get(name)?.clone())
    }

    pub fn get_offset_val(&self) -> (i32, i32) {
        (self.arg_offset, self.var_offset)
    }

    pub fn get_scope_ctype(&self) -> CType {
        self.ret_type.clone()
    }
}
