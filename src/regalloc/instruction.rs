use std::collections::HashSet;

use crate::three_ac::{operand, Instruction, Label, Operand, Set, Variant};

use super::reg_table::RegTable;

impl Instruction {
    pub fn to_code(&self, live_set: HashSet<Operand>, reg_table: &mut RegTable) -> String {
        let mut code = String::new();
        match &self.variant {
            Variant::HeaderText(l) => return format!(".section .text\nMV fp, sp\nJR {l}\nHALT\n"),
            Variant::HeaderStrings(s) => return format!(".section .strings\n{s}"),

            Variant::AddrAssign => {
                let opd = reg_table.ensure(&self.opdt, &live_set, &mut code);
                let opm = reg_table.ensure(&self.opm, &live_set, &mut code);
                if !live_set.contains(&self.opdt) {
                    reg_table.free(&opd, &live_set, &mut code);
                }
                if !live_set.contains(&self.opm) {
                    reg_table.free(&opm, &live_set, &mut code);
                }
                match self.set {
                    Set::T => code.push_str(&format!("SW {opm}, 0({opd})\n")),
                    Set::F => code.push_str(&format!("FSW {opm}, 0({opd})\n")),
                }
            }
            Variant::Assign => {
                let opm = reg_table.ensure(&self.opm, &live_set, &mut code);
                if !live_set.contains(&self.opm) {
                    reg_table.free(&opm, &live_set, &mut code);
                }
                let opd = reg_table.allocate(&self.opdt, &live_set, &mut code);
                match self.set {
                    Set::T => code.push_str(&format!("MV {opd}, {opm}\n")),
                    Set::F => code.push_str(&format!("FMV.S {opd}, {opm}\n")),
                }
                reg_table.mark_dirty(&opd, &mut code);
            }
            Variant::Free => {
                let opt = reg_table.ensure(&self.opdt, &live_set, &mut code);
                if !live_set.contains(&self.opdt) {
                    reg_table.free(&opt, &live_set, &mut code);
                }
                code.push_str(&format!("FREE {opt}\n"));
            }
            Variant::Get => {
                let opd = reg_table.allocate(&self.opdt, &live_set, &mut code);
                match self.set {
                    Set::T => code.push_str(&format!("GETI {opd}\n")),
                    Set::F => code.push_str(&format!("GETF {opd}\n")),
                }
                reg_table.mark_dirty(&opd, &mut code);
            }
            Variant::Malloc => {
                let opm = reg_table.ensure(&self.opm, &live_set, &mut code);
                if !live_set.contains(&self.opm) {
                    reg_table.free(&opm, &live_set, &mut code);
                }
                let opd = reg_table.allocate(&self.opdt, &live_set, &mut code);
                code.push_str(&format!("MALLOC {opd}, {opm}\n"));
                reg_table.mark_dirty(&opd, &mut code);
            }
            Variant::Put => {
                let opt = reg_table.ensure(&self.opdt, &live_set, &mut code);
                if !live_set.contains(&self.opdt) {
                    reg_table.free(&opt, &live_set, &mut code);
                }
                match self.set {
                    Set::T => code.push_str(&format!("PUTI {opt}\n")),
                    Set::F => code.push_str(&format!("PUTF {opt}\n")),
                }
            }
            Variant::PutS => {
                let opt = reg_table.ensure(&self.opdt, &live_set, &mut code);
                if !live_set.contains(&self.opdt) {
                    reg_table.free(&opt, &live_set, &mut code);
                }
                code.push_str(&format!("PUTS {opt}\n"));
            }
            Variant::Ret => return String::from("MV sp, fp\nLW fp, 0(fp)\nADDI sp, sp, 4\nRET\n"),
            Variant::Save => {
                let opt = reg_table.ensure(&self.opdt, &live_set, &mut code);
                if !live_set.contains(&self.opdt) {
                    reg_table.free(&opt, &live_set, &mut code);
                }
                match self.set {
                    Set::T => code.push_str(&format!("SW {opt}, 8(fp)\n")),
                    Set::F => code.push_str(&format!("FSW {opt}, 8(fp)\n")),
                }
            }
            Variant::Load(v) => {
                let opd = reg_table.allocate(&self.opdt, &live_set, &mut code);
                match self.set {
                    Set::T => code.push_str(&format!("LI {opd}, {v}\n")),
                    Set::F => code.push_str(&format!("FIMM.S {opd}, {v}\n")),
                }
                reg_table.mark_dirty(&opd, &mut code);
            }

            Variant::Address => {
                let opm = reg_table.ensure(&self.opm, &live_set, &mut code);
                if !live_set.contains(&self.opm) {
                    reg_table.free(&opm, &live_set, &mut code);
                }
                let opd = reg_table.allocate(&self.opdt, &live_set, &mut code);
                code.push_str(&format!("LW {opd}, 0({opm})\n"));
                reg_table.mark_dirty(&opd, &mut code);
            }

            Variant::Dereference => {
                let opm = reg_table.ensure(&self.opm, &live_set, &mut code);
                if !live_set.contains(&self.opm) {
                    reg_table.free(&opm, &live_set, &mut code);
                }
                let opd = reg_table.allocate(&self.opdt, &live_set, &mut code);
                match self.set {
                    Set::T => code.push_str(&format!("LW {opd}, 0({opm})\n")),
                    Set::F => code.push_str(&format!("FLW {opd}, 0({opm})\n")),
                }
                reg_table.mark_dirty(&opd, &mut code);
            }
            Variant::Reference => {
                let opd = reg_table.allocate(&self.opdt, &live_set, &mut code);
                match self.opm.variant {
                    operand::Variant::Local(i) => code.push_str(&format!("ADDI {opd}, fp, {i}\n")),
                    _ => unreachable!(),
                };
                reg_table.mark_dirty(&opd, &mut code);
            }

            Variant::Plus | Variant::Minus | Variant::Times | Variant::Divide => {
                let mut op = String::from(match self.variant {
                    Variant::Plus => "ADD",
                    Variant::Minus => "SUB",
                    Variant::Times => "MUL",
                    Variant::Divide => "DIV",
                    _ => "ERR",
                });
                if self.set == Set::F {
                    op = format!("F{op}.S");
                }

                let opm = reg_table.ensure(&self.opm, &live_set, &mut code);
                let opn = reg_table.ensure(&self.opn, &live_set, &mut code);
                if !live_set.contains(&self.opm) {
                    reg_table.free(&opm, &live_set, &mut code);
                }
                if !live_set.contains(&self.opn) {
                    reg_table.free(&opn, &live_set, &mut code);
                }
                let opd = reg_table.allocate(&self.opdt, &live_set, &mut code);
                code.push_str(&format!("{op} {opd}, {opm}, {opn}\n"));
                reg_table.mark_dirty(&opd, &mut code);
            }

            Variant::Negate => {
                let op = String::from(match self.set {
                    Set::T => "NEG",
                    Set::F => "FNEG.S",
                });

                let opm = reg_table.ensure(&self.opm, &live_set, &mut code);
                if !live_set.contains(&self.opm) {
                    reg_table.free(&opm, &live_set, &mut code);
                }
                let opd = reg_table.allocate(&self.opdt, &live_set, &mut code);
                code.push_str(&format!("{op} {opd}, {opm}\n"));
                reg_table.mark_dirty(&opd, &mut code);
            }

            Variant::Cast => {
                let op = String::from(match self.set {
                    Set::T => "FMOVI.S",
                    Set::F => "IMOVF.S",
                });

                let opm = reg_table.ensure(&self.opm, &live_set, &mut code);
                if !live_set.contains(&self.opm) {
                    reg_table.free(&opm, &live_set, &mut code);
                }
                let opd = reg_table.allocate(&self.opdt, &live_set, &mut code);
                code.push_str(&format!("{op} {opd}, {opm}\n"));
                reg_table.mark_dirty(&opd, &mut code);
            }

            Variant::Equal(l)
            | Variant::NotEqual(l)
            | Variant::Less(l)
            | Variant::LessEqual(l)
            | Variant::Greater(l)
            | Variant::GreaterEqual(l) => match self.set {
                Set::T => {
                    let opm = reg_table.ensure(&self.opm, &live_set, &mut code);
                    let opn = reg_table.ensure(&self.opn, &live_set, &mut code);
                    if !live_set.contains(&self.opm) {
                        reg_table.free(&opm, &live_set, &mut code);
                    }
                    if !live_set.contains(&self.opn) {
                        reg_table.free(&opn, &live_set, &mut code);
                    }
                    code.push_str(&reg_table.spill_registers());
                    code.push_str(&format!(
                        "{} {opm}, {opn}, {l}\n",
                        match self.variant {
                            Variant::Equal(_) => "BNE",
                            Variant::NotEqual(_) => "BEQ",
                            Variant::Less(_) => "BGE",
                            Variant::LessEqual(_) => "BGT",
                            Variant::Greater(_) => "BLE",
                            Variant::GreaterEqual(_) => "BLT",
                            _ => "ERR",
                        }
                    ));
                }
                Set::F => {
                    let opm = reg_table.ensure(&self.opm, &live_set, &mut code);
                    let opn = reg_table.ensure(&self.opn, &live_set, &mut code);
                    if !live_set.contains(&self.opm) {
                        reg_table.free(&opm, &live_set, &mut code);
                    }
                    if !live_set.contains(&self.opn) {
                        reg_table.free(&opn, &live_set, &mut code);
                    }
                    let opd = reg_table.allocate(&self.opdt, &live_set, &mut code);
                    reg_table.mark_dirty(&opd, &mut code);
                    let (op1, op2) = match self.variant {
                        Variant::Equal(_) => (String::from("FEQ.S"), String::from("BEQ")),
                        Variant::NotEqual(_) => (String::from("FEQ.S"), String::from("BNE")),
                        Variant::Less(_) => (String::from("FLT.S"), String::from("BEQ")),
                        Variant::LessEqual(_) => (String::from("FLE.S"), String::from("BEQ")),
                        Variant::Greater(_) => (String::from("FLE.S"), String::from("BNE")),
                        Variant::GreaterEqual(_) => (String::from("FLT.S"), String::from("BNE")),
                        _ => (String::from("ERR"), String::from("ERR")),
                    };
                    code.push_str(&reg_table.spill_registers());
                    code.push_str(&format!(
                        "{} {opd}, {opm}, {opn}\n{} {opd}, x0, {l}\n",
                        op1, op2,
                    ));
                }
            },

            Variant::Label(l) => {
                return match l {
                    Label::FunctionHead(_) => {
                        format!("{l}:\nADDI sp, sp, -4\nSW fp, 0(sp)\nMV fp, sp\n")
                    }
                    _ => format!("{l}:\n"),
                }
            }
            Variant::Jump(l) => return format!("J {l}\n"),
            Variant::Call(l, o) => {
                let total_offset = (o.len() + 2) * 4;
                code.push_str(&format!("ADDI sp, sp, -{total_offset}\nSW ra, 0(sp)\n"));

                let mut offset = 8;
                for arg in o {
                    let op = reg_table.ensure(arg, &live_set, &mut code);
                    if !live_set.contains(arg) {
                        reg_table.free(&op, &live_set, &mut code);
                    }
                    code.push_str(&format!(
                        "{} {op}, {offset}(sp)\n",
                        match arg.otype {
                            operand::Type::T => "SW",
                            operand::Type::F => "FSW",
                        }
                    ));
                    offset += 4;
                }

                code.push_str(&reg_table.spill_registers());
                code.push_str(&format!("JR {l}\nLW ra, 0(sp)\n"));
                if self.opdt.variant != operand::Variant::Null {
                    let opd = reg_table.allocate(&self.opdt, &live_set, &mut code);
                    code.push_str(&format!(
                        "{} {opd}, 4(sp)\n",
                        match self.set {
                            Set::T => "LW",
                            Set::F => "FLW",
                        }
                    ));
                    reg_table.mark_dirty(&opd, &mut code);
                }
                code.push_str(&format!("ADDI sp, sp, {total_offset}\n"));
            }

            Variant::Alloc(v) => return format!("ADDI sp, sp, -{v}\n"),
            Variant::SpillRegisters => return reg_table.spill_registers(),
        }

        code
    }
}
