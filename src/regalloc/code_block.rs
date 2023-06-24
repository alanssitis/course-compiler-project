use std::collections::{HashSet, VecDeque};

use crate::three_ac::{Instruction, Operand, Variant};

pub fn next_from_instructions(
    instructions: &mut VecDeque<Instruction>,
) -> Option<(Vec<Instruction>, HashSet<Operand>)> {
    let mut collection = Vec::new();
    let mut live_set = HashSet::new();

    while let Some(i) = instructions.pop_front() {
        if matches!(&i.variant, Variant::Label(_)) && !collection.is_empty() {
            instructions.push_front(i);
            collection.push(Instruction::spill_registers());
            break;
        }

        i.add_operands_to_set(&mut live_set);
        if matches!(
            &i.variant,
            Variant::HeaderText(_)
                | Variant::HeaderStrings(_)
                | Variant::Ret
                | Variant::Equal(_)
                | Variant::NotEqual(_)
                | Variant::Less(_)
                | Variant::LessEqual(_)
                | Variant::Greater(_)
                | Variant::GreaterEqual(_)
                | Variant::Jump(_)
                | Variant::Call(..)
        ) {
            if matches!(&i.variant, Variant::Ret | Variant::Jump(_)) {
                collection.push(Instruction::spill_registers());
            }
            collection.push(i);
            break;
        }
        collection.push(i);
    }
    if collection.is_empty() {
        return None;
    }
    Some((collection, live_set))
}

impl Instruction {
    fn add_operands_to_set(&self, set: &mut HashSet<Operand>) {
        self.opdt.insert_to_set_if_a_var(set);
        self.opm.insert_to_set_if_a_var(set);
        self.opn.insert_to_set_if_a_var(set);
    }
}

impl Operand {
    pub fn insert_to_set_if_a_var(&self, set: &mut HashSet<Operand>) {
        if self.is_variable() {
            self.insert_to_set(set);
        }
    }

    pub fn insert_to_set(&self, set: &mut HashSet<Operand>) {
        if self.is_spillable() {
            set.insert(*self);
        }
    }

    pub fn remove_from_set(&self, set: &mut HashSet<Operand>) {
        if self.is_spillable() {
            set.remove(self);
        }
    }
}
