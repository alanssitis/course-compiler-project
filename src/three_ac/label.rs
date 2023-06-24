use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Label {
    FunctionHead(i32),
    FunctionTail(i32),

    BlockBranch(u32),
    BlockJump(u32),
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::FunctionHead(n) => write!(f, "func_{n}"),
            Self::FunctionTail(n) => write!(f, "func_tail_{n}"),

            Self::BlockBranch(n) => write!(f, "branch_{n}"),
            Self::BlockJump(n) => write!(f, "jump_{n}"),
        }
    }
}
