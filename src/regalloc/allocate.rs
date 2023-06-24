use crate::error::Error;
use crate::three_ac::Instructions;

use super::code_block;
use super::liveness_analysis;
use super::reg_table::RegTable;

pub fn from_instructions(instructions: Instructions, reg_count: u32) -> Result<String, Error> {
    let mut instrs = instructions.instructions;
    let mut output = String::new();
    let mut reg_table = RegTable::new(reg_count)?;

    while let Some((block, mut live_set)) = code_block::next_from_instructions(&mut instrs) {
        let analyzed = liveness_analysis::for_codeblock(block, &mut live_set);
        for (i, l) in analyzed {
            // println!("{i} {l:?}");
            output.push_str(&i.to_code(l.clone(), &mut reg_table));
        }
    }

    Ok(output)
}
