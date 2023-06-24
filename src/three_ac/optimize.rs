use std::collections::VecDeque;

use super::instruction::Variant;
use super::instructions::Instructions;
use super::operand;

impl Instructions {
    pub fn optimize(self) -> Self {
        self.peephole()
    }

    fn peephole(self) -> Self {
        let mut list = self.instructions;
        let mut instructions = VecDeque::new();

        let mut curr = match list.pop_front() {
            Some(i) => i,
            None => {
                return Self {
                    instructions,
                    tmp: None,
                }
            }
        };

        while let Some(next) = list.pop_front() {
            match next.variant {
                Variant::Assign => {
                    if next.opm == curr.opdt
                        && matches!(
                            next.opm.variant,
                            operand::Variant::Temp(_) | operand::Variant::TempFloat(_)
                        )
                    {
                        curr.opdt = next.opdt;
                        continue;
                    }
                }
                Variant::Label(l) => {
                    if Variant::Jump(l) == curr.variant {
                        curr = next;
                        continue;
                    }
                }
                _ => {}
            }
            instructions.push_back(curr);
            curr = next;
        }
        instructions.push_back(curr);

        Self {
            instructions,
            tmp: None,
        }
    }
}
