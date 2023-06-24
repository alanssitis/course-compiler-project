use pest::iterators::Pairs;
use pest::pratt_parser::PrattParser;

use crate::error::Error;
use crate::parser::Rule;

lazy_static::lazy_static! {
    static ref TYPE_CLIMBER: PrattParser<Rule> = {
        use pest::pratt_parser::Op;

        PrattParser::new()
            .op(Op::postfix(Rule::ptr))
    };
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CType {
    Int,
    Float,
    Str,
    Ptr(Box<Self>),
    Void,
}

impl CType {
    pub fn is_mutable(&self) -> bool {
        matches!(self, CType::Int | CType::Float | CType::Ptr(_))
    }

    pub fn from_base_type(pairs: Pairs<Rule>) -> Self {
        TYPE_CLIMBER
            .map_primary(|p| match p.as_rule() {
                Rule::int => Self::Int,
                Rule::float => Self::Float,
                Rule::void => Self::Void,
                _ => unreachable!("from_base_type: expected base_type, found other"),
            })
            .map_postfix(|lhs, op| match op.as_rule() {
                Rule::ptr => Self::Ptr(Box::new(lhs)),
                _ => unreachable!("from_base_type: expected ptr, found other"),
            })
            .parse(pairs)
    }

    pub fn dereference(self) -> Result<Self, Error> {
        match self {
            Self::Ptr(t) => Ok(*t),
            _ => Err(Error::Type),
        }
    }
}
