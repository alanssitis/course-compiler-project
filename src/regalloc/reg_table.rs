use std::collections::{BTreeMap, HashSet};
use std::fmt;

use crate::error::Error;
use crate::three_ac::{operand, Operand};

#[derive(Eq, PartialEq)]
pub enum Register {
    X(Regular),
    F(Float),
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::X(r) => write!(f, "{r}"),
            Self::F(r) => write!(f, "{r}"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Regular(u32);

impl fmt::Display for Regular {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "x{}", self.0)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Float(u32);

impl fmt::Display for Float {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "f{}", self.0)
    }
}

#[derive(Clone, Debug)]
struct Entry {
    operand: Operand,
    dirty: bool,
}

impl Entry {
    fn new_null() -> Self {
        Self {
            operand: Operand::new_null(),
            dirty: false,
        }
    }

    fn reset(&mut self) {
        self.operand = Operand::new_null();
        self.dirty = false;
    }

    fn dirty(&mut self) {
        self.dirty = true;
    }

    fn update(&mut self, operand: Operand) {
        self.operand = operand;
        self.dirty = false;
    }

    fn is_null(&self) -> bool {
        self.operand.variant == operand::Variant::Null
    }

    fn spill_regular_entry(&self, r: Regular) -> String {
        match &self.operand.variant {
            operand::Variant::Global(a) => {
                format!("LA x3, 0x{a:08x}\nSW {r}, 0(x3)\n")
            }
            operand::Variant::Local(i) => format!("SW {r}, {i}(fp)\n"),
            _ => String::new(),
        }
    }

    fn spill_float_entry(&self, r: Float) -> String {
        match &self.operand.variant {
            operand::Variant::Global(a) => {
                format!("LA x3, 0x{a:08x}\nFSW {r}, 0(x3)\n")
            }
            operand::Variant::Local(i) => format!("FSW {r}, {i}(fp)\n"),
            _ => String::new(),
        }
    }
}

#[derive(Debug)]
pub struct RegTable {
    regular: BTreeMap<Regular, Entry>,
    float: BTreeMap<Float, Entry>,
}

impl RegTable {
    pub fn new(reg_count: u32) -> Result<Self, Error> {
        if reg_count < 8 {
            return Err(Error::RegAlloc(format!(
                "new: reg_count {reg_count} less than 8"
            )));
        }
        let mut regular = (4..reg_count)
            .map(|u| (Regular(u), Entry::new_null()))
            .collect::<BTreeMap<Regular, Entry>>();
        regular.remove(&Regular(8));
        let float = (1..reg_count)
            .map(|u| (Float(u), Entry::new_null()))
            .collect::<BTreeMap<Float, Entry>>();
        Ok(Self { regular, float })
    }

    pub fn spill_registers(&mut self) -> String {
        let mut out = String::new();

        for (r, entry) in self.regular.iter_mut() {
            if entry.dirty {
                out.push_str(&entry.spill_regular_entry(*r));
            }
            entry.reset();
        }
        for (r, entry) in self.float.iter_mut() {
            if entry.dirty {
                out.push_str(&entry.spill_float_entry(*r));
            }
            entry.reset();
        }

        out
    }

    pub fn ensure(&mut self, op: &Operand, set: &HashSet<Operand>, code: &mut String) -> Register {
        if let Operand {
            variant: operand::Variant::Str(a),
            ..
        } = op
        {
            code.push_str(&format!("LA x3, 0x{a:08x}\n"));
            return Register::X(Regular(3));
        }
        match op.otype {
            operand::Type::T => {
                for (reg, Entry { operand, .. }) in &self.regular {
                    if operand == op {
                        return Register::X(*reg);
                    }
                }
                let r = self.allocate(op, set, code);
                match &op.variant {
                    operand::Variant::Global(a) => {
                        code.push_str(&format!("LA x3, 0x{a:08x}\nLW {r}, 0(x3)\n"))
                    }
                    operand::Variant::Local(i) => code.push_str(&format!("LW {r}, {i}(fp)\n")),
                    _ => return Register::X(Regular(99)),
                }
                r
            }
            operand::Type::F => {
                for (reg, Entry { operand, .. }) in &self.float {
                    if operand == op {
                        return Register::F(*reg);
                    }
                }
                let r = self.allocate(op, set, code);
                match &op.variant {
                    operand::Variant::Global(i) => {
                        code.push_str(&format!("LA x3, 0x{i:08x}\nFLW {r}, 0(x3)\n"))
                    }
                    operand::Variant::Local(i) => code.push_str(&format!("FLW {r}, {i}(fp)\n")),
                    _ => return Register::F(Float(99)),
                }
                r
            }
        }
    }

    pub fn allocate(
        &mut self,
        op: &Operand,
        set: &HashSet<Operand>,
        code: &mut String,
    ) -> Register {
        if let Some(reg) = self.choose_register(*op) {
            self.free(&reg, set, code);
            match reg {
                Register::X(r) => {
                    if let Some(entry) = self.regular.get_mut(&r) {
                        entry.update(*op);
                    }
                }
                Register::F(f) => {
                    if let Some(entry) = self.float.get_mut(&f) {
                        entry.update(*op);
                    }
                }
            }
            return reg;
        }

        Register::X(Regular(0))
    }

    fn choose_register(&self, operand: Operand) -> Option<Register> {
        match operand.otype {
            operand::Type::T => {
                if let Some((reg, _)) = self.regular.iter().reduce(|(r, e), (reg, entry)| {
                    if e.operand == operand {
                        return (r, e);
                    } else if entry.operand == operand {
                        return (reg, entry);
                    }
                    if !e.is_null() && entry.is_null() || (e.dirty && !entry.dirty) {
                        return (reg, entry);
                    }
                    (r, e)
                }) {
                    return Some(Register::X(*reg));
                }
            }
            operand::Type::F => {
                if let Some((reg, _)) = self.float.iter().reduce(|(r, e), (reg, entry)| {
                    if e.operand == operand {
                        return (r, e);
                    } else if entry.operand == operand {
                        return (reg, entry);
                    }
                    if !e.is_null() && entry.is_null() || (e.dirty && !entry.dirty) {
                        return (reg, entry);
                    }
                    (r, e)
                }) {
                    return Some(Register::F(*reg));
                }
            }
        }
        None
    }

    pub fn free(&mut self, r: &Register, set: &HashSet<Operand>, code: &mut String) {
        match r {
            Register::X(x) => {
                if let Some(entry) = self.regular.get_mut(x) {
                    if entry.dirty && set.contains(&entry.operand) {
                        code.push_str(&entry.spill_regular_entry(*x));
                    }
                    entry.reset();
                }
            }
            Register::F(f) => {
                if let Some(entry) = self.float.get_mut(f) {
                    if entry.dirty && set.contains(&entry.operand) {
                        match entry.operand.variant {
                            operand::Variant::Global(i) => {
                                code.push_str(&format!("LA x3, 0x{i:08x}\nFSW {r}, 0(x3)\n"))
                            }
                            operand::Variant::Local(i) => {
                                code.push_str(&format!("FSW {r}, {i}(fp)\n"))
                            }
                            _ => return,
                        }
                    }
                    entry.reset();
                }
            }
        }
    }

    pub fn mark_dirty(&mut self, r: &Register, code: &mut String) {
        match r {
            Register::X(x) => {
                if let Some(entry) = self.regular.get_mut(x) {
                    entry.dirty();
                }
            }
            Register::F(f) => {
                if let Some(entry) = self.float.get_mut(f) {
                    entry.dirty();
                }
            }
        }
        code.push_str(&self.spill_aliased_registers());
    }

    fn spill_aliased_registers(&mut self) -> String {
        let mut out = String::new();

        for (reg, entry) in self.regular.iter_mut() {
            if matches!(
                entry.operand.variant,
                operand::Variant::Global(_) | operand::Variant::Local(_)
            ) {
                if entry.dirty {
                    out.push_str(&entry.spill_regular_entry(*reg));
                }
                entry.reset();
            }
        }

        for (flt, entry) in self.float.iter_mut() {
            if matches!(
                entry.operand.variant,
                operand::Variant::Global(_) | operand::Variant::Local(_)
            ) {
                if entry.dirty {
                    out.push_str(&entry.spill_float_entry(*flt));
                }
                entry.reset();
            }
        }

        out
    }
}

impl fmt::Display for RegTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut out = String::new();
        for (k, e) in &self.regular {
            if e.operand.variant != operand::Variant::Null {
                out.push_str(&format!("{}: {}\n", Register::X(*k), e.operand));
            }
        }
        for (k, e) in &self.float {
            if e.operand.variant != operand::Variant::Null {
                out.push_str(&format!("{}: {}\n", Register::F(*k), e.operand));
            }
        }

        write!(f, "{out}")
    }
}
