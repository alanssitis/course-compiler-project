use crate::ast;
use crate::error::Error;
use crate::regalloc;
use crate::symtable::SymTable;
use crate::three_ac::{Count, Instructions};

pub fn generate_code(
    ast: ast::Node,
    mut symtable: SymTable,
    reg_count: u32,
) -> Result<String, Error> {
    let instructions = get_instructions(ast, &mut symtable)?;
    // println!("{}", instructions);
    regalloc::from_instructions(instructions, reg_count)
}

fn get_instructions(ast: ast::Node, symtable: &mut SymTable) -> Result<Instructions, Error> {
    let instrs = Instructions::from_ast(
        ast,
        &mut Count {
            regular: 0,
            float: 0,
            label: 0,
        },
        symtable,
    )?
    .add_headers(
        symtable.get_symbol(&String::from("main"))?.address(),
        symtable.strings_in_asm(),
    )
    .optimize();

    Ok(instrs)
}
