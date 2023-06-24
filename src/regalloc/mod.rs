pub use self::allocate::from_instructions;

mod allocate;
mod code_block;
mod instruction;
mod liveness_analysis;
mod reg_table;
