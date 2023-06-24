pub use self::instruction::{Instruction, Set, Variant};
pub use self::instructions::{Count, Instructions};
pub use self::label::Label;
pub use self::operand::Operand;

mod instruction;
mod instructions;
mod label;
pub mod operand;
mod optimize;
