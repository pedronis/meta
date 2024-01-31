use mparse::ParseableInstr;
use std::collections::HashMap;
use std::error::Error;

const PRINT_AREA_SIZE: usize = 100;
const EPS: f64 = 0.000001;

#[derive(Debug)]
pub struct M {
    mem: HashMap<u32, f64>,
    stack: Vec<f64>,
    print_area: String,
}

impl M {
    pub fn new() -> Self {
        M {
            mem: HashMap::new(),
            stack: Vec::new(),
            print_area: String::with_capacity(PRINT_AREA_SIZE),
        }
    }

    fn push(&mut self, u: f64) {
        self.stack.push(u);
    }

    fn pop(&mut self) -> f64 {
        self.stack.pop().expect("machine stack underflow")
    }

    fn ld(&mut self, loc: u32) {
        if let Some(v) = self.mem.get(&loc) {
            self.push(*v)
        } else {
            self.push(0.0)
        }
    }

    fn st(&mut self, loc: u32) {
        let v = self.pop();
        self.mem.insert(loc, v);
    }

    fn add(&mut self) {
        let a = self.pop();
        let b = self.pop();
        self.push(a + b);
    }

    fn mlt(&mut self) {
        let a = self.pop();
        let b = self.pop();
        self.push(a * b);
    }

    fn equ(&mut self) {
        let a = self.pop();
        let b = self.pop();
        let mut fl = 0.0;
        if (a - b).abs() < EPS {
            fl = 1.0
        }
        self.push(fl);
    }

    fn sub(&mut self) {
        let a = self.pop();
        let b = self.pop();
        self.push(a - b);
    }

    fn edt(&mut self, s: &str) {
        let n = self.pop().round();
        if n < 0.0 {
            return;
        }
        let start = n as usize;
        let sz = s.len();
        if start + sz > PRINT_AREA_SIZE {
            return;
        }
        if self.print_area.is_empty() {
            self.print_area.push_str(&" ".repeat(PRINT_AREA_SIZE));
        }
        self.print_area.replace_range(start..start + sz, s);
    }

    fn pnt(&mut self) {
        println!("{}", self.print_area.trim_end());
        self.print_area.truncate(0);
    }

    pub fn execute(&mut self, pgm: &mparse::MProgram<MInstr>) {
        let mut ic: usize = 0;
        loop {
            match &pgm.instrs[ic] {
                MInstr::Undef => panic!("Undef unexpected in program"),
                MInstr::LDL(v) => {
                    self.push(*v);
                }
                MInstr::LD(_, loc) => self.ld(*loc),
                MInstr::ST(_, loc) => self.st(*loc),
                MInstr::B(_, jic) => {
                    ic = *jic;
                    continue;
                }
                MInstr::BFP(_, jic) => {
                    if self.pop() == 0.0 {
                        ic = *jic;
                        continue;
                    }
                }
                MInstr::BTP(_, jic) => {
                    if self.pop() != 0.0 {
                        ic = *jic;
                        continue;
                    }
                }
                MInstr::ADD => self.add(),
                MInstr::SUB => self.sub(),
                MInstr::MLT => self.mlt(),
                MInstr::EQU => self.equ(),
                MInstr::HLT => break,
                MInstr::EDT(s) => self.edt(s),
                MInstr::PNT => self.pnt(),
            }
            ic += 1;
        }
    }
}

#[derive(Debug)]
pub enum MInstr {
    // branch
    B(String, usize),
    BFP(String, usize),
    BTP(String, usize),
    // constant
    LDL(f64),
    // memory
    ST(String, u32),
    LD(String, u32),
    // operations
    EQU,
    ADD,
    MLT,
    SUB,
    EDT(String),
    PNT,
    HLT,
    Undef,
}

impl ParseableInstr for MInstr {
    const UNDEF: Self = MInstr::Undef;
    const ACCEPT_BLK: bool = true;

    fn is_undefined(&self) -> bool {
        matches!(self, MInstr::Undef)
    }

    fn with_label(ins: &str, label: String) -> Self {
        match ins {
            "B" => MInstr::B(label, 0),
            "ST" => MInstr::ST(label, 0),
            "LD" => MInstr::LD(label, 0),
            "BTP" => MInstr::BTP(label, 0),
            "BFP" => MInstr::BFP(label, 0),
            _ => MInstr::Undef,
        }
    }

    fn with_num(ins: &str, n: f64) -> Self {
        match ins {
            "LDL" => MInstr::LDL(n),
            _ => MInstr::Undef,
        }
    }

    fn with_string(ins: &str, s: String) -> Self {
        match ins {
            "EDT" => MInstr::EDT(s),
            _ => MInstr::Undef,
        }
    }

    fn with_noarg(ins: &str) -> Self {
        match ins {
            "EQU" => MInstr::EQU,
            "ADD" => MInstr::ADD,
            "SUB" => MInstr::SUB,
            "MLT" => MInstr::MLT,
            "PNT" => MInstr::PNT,
            "HLT" => MInstr::HLT,
            _ => MInstr::Undef,
        }
    }

    fn aaa_of(&self) -> mparse::AAAUse {
        match self {
            MInstr::ST(aaa, _) | MInstr::LD(aaa, _) => mparse::AAAUse::Mem(aaa.to_string()),
            MInstr::B(aaa, _) | MInstr::BFP(aaa, _) | MInstr::BTP(aaa, _) => {
                mparse::AAAUse::IC(aaa.to_string())
            }
            _ => mparse::AAAUse::None,
        }
    }

    fn reconstruct_with_addr(&mut self, aaa: String, addr: u32) {
        *self = match self {
            MInstr::ST(_, _) => MInstr::ST(aaa, addr),
            MInstr::LD(_, _) => MInstr::LD(aaa, addr),
            _ => panic!("internal error: unknown aaa instruction"),
        }
    }

    fn reconstruct_with_ic(&mut self, aaa: String, ic: usize) {
        *self = match self {
            MInstr::B(_, _) => MInstr::B(aaa, ic),
            MInstr::BFP(_, _) => MInstr::BFP(aaa, ic),
            MInstr::BTP(_, _) => MInstr::BTP(aaa, ic),
            _ => panic!("internal error: unknown aaa instruction"),
        };
    }
}

pub fn run(opts: Options) -> Result<(), Box<dyn Error>> {
    let p = mparse::load::<MInstr>(&opts.pgm_path)?;
    println!("{p:#?}");
    let mut m = M::new();
    m.execute(&p);
    Ok(())
}

pub struct Options {
    pub pgm_path: String,
}

impl Options {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Self, &'static str> {
        args.next();
        let pgm_path = match args.next() {
            Some(arg) => arg,
            None => return Err("missing program path argument"),
        };
        Ok(Options { pgm_path })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_vs_lexing() {
        assert!(mparse::parse::<MInstr>(
            r#"
  # comment
# comment
 B  A # jump
X#ok
   BLK 003#blk
A  # label
   LDL  5.0
 ST X
   LD X
   EDT'233'
   END#comment
"#
        )
        .is_ok());
    }

    #[test]
    fn m() {
        let mut m = M::new();
        m.push(1.0);
        assert_eq!(m.pop(), 1.0);
    }

    #[test]
    fn m_add() {
        let mut m = M::new();
        m.push(2.0);
        m.push(3.0);
        m.add();
        assert_eq!(m.pop(), 5.0);
    }

    #[test]
    fn m_mlt() {
        let mut m = M::new();
        m.push(3.0);
        m.push(-4.0);
        m.mlt();
        assert_eq!(m.pop(), -12.0);
    }

    #[test]
    fn m_equ() {
        let mut m = M::new();
        m.push(3.0);
        m.push(-4.0);
        m.mlt();
        m.push(-12.0);
        m.equ();
        assert_eq!(m.pop(), 1.0);
    }

    #[test]
    fn m_edt_simple() {
        let mut m = M::new();
        m.push(3.0);
        m.edt("abc");
        assert_eq!(m.print_area.len(), PRINT_AREA_SIZE);
        assert_eq!(m.print_area.trim_end(), "   abc");
        m.push(99.0);
        m.edt("z");
        assert_eq!(m.print_area.len(), PRINT_AREA_SIZE);
        assert_eq!(m.print_area.trim_end(), "   abc                                                                                             z");
        m.push(100.0);
        m.edt("x");
        assert_eq!(m.print_area.trim_end(), "   abc                                                                                             z");
        m.push(98.0);
        m.edt("xy");
        assert_eq!(m.print_area.trim_end(), "   abc                                                                                            xy");
        m.push(98.0);
        m.edt("zzz");
        assert_eq!(m.print_area.trim_end(), "   abc                                                                                            xy");
        m.push(4.0);
        m.edt("x");
        m.push(6.0);
        m.edt("y");
        assert_eq!(m.print_area.trim_end(), "   axcy                                                                                           xy");
        m.push(-1.0);
        m.edt("aa");
        assert_eq!(m.print_area.trim_end(), "   axcy                                                                                           xy");
        m.push(0.0);
        m.edt("aa");
        assert_eq!(m.print_area.trim_end(), "aa axcy                                                                                           xy");
        // printing
        m.pnt();
        assert_eq!(m.print_area, "");
        // further
        m.push(0.0);
        m.edt("aa");
        assert_eq!(m.print_area.trim_end(), "aa");
    }

    #[test]
    fn m_st_ld_sub() {
        let mut m = M::new();
        m.ld(0);
        let v = m.pop();
        assert_eq!(v, 0.0);
        m.push(2.0);
        m.st(0);
        m.push(3.0);
        m.st(1);
        m.ld(1);
        m.ld(0);
        m.sub();
        assert_eq!(m.pop(), -1.0);
    }
}
