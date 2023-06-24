use std::collections::{HashSet, VecDeque};

use crate::three_ac::{Instruction, Operand, Variant};

pub fn for_codeblock(
    mut block: Vec<Instruction>,
    set: &mut HashSet<Operand>,
) -> VecDeque<(Instruction, HashSet<Operand>)> {
    let mut out = VecDeque::new();

    while let Some(i) = block.pop() {
        let v = i.variant.clone();
        let s = set.clone();
        match v {
            Variant::AddrAssign => {
                i.opdt.insert_to_set(set);
                i.opm.insert_to_set(set);
            }
            Variant::Assign | Variant::Malloc | Variant::Negate | Variant::Cast => {
                i.opdt.remove_from_set(set);
                i.opm.insert_to_set(set);
            }
            Variant::Get | Variant::Load(_) => {
                i.opdt.remove_from_set(set);
            }
            Variant::Address | Variant::Dereference | Variant::Reference => {
                i.opdt.remove_from_set(set);
                i.opm.insert_to_set(set);
            }
            Variant::Put | Variant::PutS | Variant::Ret | Variant::Save | Variant::Free => {
                i.opdt.insert_to_set(set);
            }

            Variant::Plus | Variant::Minus | Variant::Times | Variant::Divide => {
                i.opdt.remove_from_set(set);
                i.opm.insert_to_set(set);
                i.opn.insert_to_set(set);
            }

            Variant::Equal(_)
            | Variant::NotEqual(_)
            | Variant::Less(_)
            | Variant::LessEqual(_)
            | Variant::Greater(_)
            | Variant::GreaterEqual(_) => {
                i.opm.insert_to_set(set);
                i.opn.insert_to_set(set);
            }

            Variant::Call(_, ops) => {
                i.opdt.remove_from_set(set);
                for o in ops {
                    o.insert_to_set(set);
                }
            }

            Variant::HeaderText(_)
            | Variant::HeaderStrings(_)
            | Variant::Label(_)
            | Variant::Jump(_)
            | Variant::Alloc(_)
            | Variant::SpillRegisters => {}
        }
        out.push_front((i, s));
    }

    out
}
