use std::error::Error;
use std::fs;

use mparse::AAAUse;
use mparse::ParseableInstr;

#[derive(Debug)]
pub enum Recognition {
    Recognized,
    Unrecognized,
}

pub use Recognition::*;

#[derive(Debug)]
pub enum SynError {
    Unexpected,
}

pub type MResult = Result<Recognition, SynError>;

#[derive(Debug)]
pub struct M<'a> {
    input: &'a str,
    pos: usize,
    sw: bool,
    last: &'a str,
    a_cnt: u16,
    b_cnt: u16,
    output: String,
    stk: Vec<MStackVal>,
}

#[derive(Debug)]
enum MStackVal {
    Lb(String),
    Back { ric: usize, blanks: bool },
}

impl<'a> M<'a> {
    pub fn new(input: &'a str) -> Self {
        M {
            input,
            pos: 0,
            sw: false,
            last: "",
            a_cnt: 0,
            b_cnt: 0,
            output: " ".repeat(8),
            stk: Vec::new(),
        }
    }

    fn eat_ws(&mut self) {
        let mut rest = &self.input[self.pos..];
        while !rest.is_empty() && rest.chars().next().unwrap().is_ascii_whitespace() {
            self.pos += 1;
            rest = &self.input[self.pos..];
        }
    }

    pub fn tst(&mut self, s: &str) -> bool {
        self.eat_ws();
        let rest = &self.input[self.pos..];
        if rest.starts_with(s) {
            let start = self.pos;
            self.pos += s.len();
            self.last = &self.input[start..self.pos];
            self.sw = true
        } else {
            self.sw = false
        }
        self.sw
    }

    pub fn id(&mut self) -> bool {
        self.eat_ws();
        let mut rest = &self.input[self.pos..];
        self.sw = false;
        if rest.is_empty() || !rest.chars().next().unwrap().is_ascii_alphabetic() {
            return false;
        }
        self.sw = true;
        let start = self.pos;
        while !rest.is_empty() && rest.chars().next().unwrap().is_ascii_alphanumeric() {
            self.pos += 1;
            rest = &self.input[self.pos..];
        }
        self.last = &self.input[start..self.pos];
        true
    }

    pub fn num(&mut self) -> bool {
        self.eat_ws();
        let mut rest = &self.input[self.pos..];
        self.sw = false;
        if rest.is_empty() || !rest.chars().next().unwrap().is_ascii_digit() {
            return false;
        }
        let start = self.pos;
        let mut end = self.pos;
        while !rest.is_empty() {
            let cand_digit = rest.chars().next().unwrap();
            if cand_digit != '.' && !cand_digit.is_ascii_digit() {
                break;
            }
            end += 1;
            rest = &self.input[end..];
        }
        let num = &self.input[start..end];
        if num.ends_with('.') || num.contains("..") {
            return false;
        }
        self.pos = end;
        self.sw = true;
        self.last = num;
        true
    }

    pub fn sr(&mut self) -> bool {
        self.eat_ws();
        let mut rest = &self.input[self.pos..];
        self.sw = false;
        if !rest.starts_with('\'') {
            return false;
        }
        let start = self.pos;
        let mut end = self.pos;
        loop {
            end += 1;
            rest = &self.input[end..];
            if rest.is_empty() {
                break;
            }
            if rest.starts_with('\'') {
                end += 1;
                break;
            }
        }
        let sr = &self.input[start..end];
        if !sr.ends_with('\'') {
            return false;
        }
        self.sw = true;
        self.last = sr;
        self.pos = end;
        true
    }

    pub fn cll(&mut self, ric: usize) {
        let stk_sz = self.stk.len();
        let mut blanks = false;
        if stk_sz >= 2 {
            match &self.stk.as_slice()[stk_sz - 2..] {
                [MStackVal::Lb(l1), MStackVal::Lb(l2)] if l1.is_empty() && l2.is_empty() => {
                    blanks = true;
                    self.stk.drain(stk_sz - 2..);
                }
                _ => (),
            }
        }
        self.stk.push(MStackVal::Back { ric, blanks });
        self.stk.push(MStackVal::Lb("".to_string()));
        self.stk.push(MStackVal::Lb("".to_string()));
    }

    pub fn r(&mut self) -> usize {
        let stk_sz = self.stk.len();
        if stk_sz >= 3 {
            if let MStackVal::Back { ric, blanks } = self.stk[stk_sz - 3] {
                self.stk.drain(stk_sz - 3..);
                if blanks {
                    self.stk.push(MStackVal::Lb("".to_string()));
                    self.stk.push(MStackVal::Lb("".to_string()));
                }
                return ric;
            }
        }
        panic!("machine state stack unmatched return")
    }

    pub fn set(&mut self) {
        self.sw = true;
    }

    pub fn be(&self) -> MResult {
        if !self.sw {
            return Err(SynError::Unexpected);
        }
        Ok(Recognized)
    }

    pub fn cl(&mut self, s: &str) {
        self.output.push_str(s);
        self.output.push(' ');
    }

    pub fn ci(&mut self) {
        if self.sw {
            self.output.push_str(self.last);
        }
    }

    pub fn gn1(&mut self) {
        let stk_sz = self.stk.len();
        if stk_sz >= 2 {
            let newlb: String;
            if let MStackVal::Lb(s) = &mut self.stk[stk_sz - 2] {
                if s.is_empty() {
                    self.a_cnt += 1;
                    newlb = format!("A{:03}", self.a_cnt);
                    s.push_str(&newlb);
                } else {
                    newlb = s.clone();
                }
                self.output.push_str(&newlb);
                self.output.push(' ');
                return;
            }
        }
        panic!("malformed machine state stack")
    }

    pub fn gn2(&mut self) {
        let stk_sz = self.stk.len();
        if stk_sz >= 1 {
            let newlb: String;
            if let MStackVal::Lb(s) = &mut self.stk[stk_sz - 1] {
                if s.is_empty() {
                    self.b_cnt += 1;
                    newlb = format!("B{:03}", self.b_cnt);
                    s.push_str(&newlb);
                } else {
                    newlb = s.clone();
                }
                self.output.push_str(&newlb);
                self.output.push(' ');
                return;
            }
        }
        panic!("malformed machine state stack")
    }

    pub fn out(&mut self) {
        self.output.push('\n');
        self.output.push_str(&" ".repeat(8));
    }

    pub fn lb(&mut self) {
        if let Some(nl) = self.output.rfind('\n') {
            self.output.truncate(nl + 1);
        } else {
            self.output.truncate(0);
        }
    }

    pub fn left(&self) -> String {
        self.input[self.pos..].trim_start().to_string()
    }

    pub fn generated(&self) -> Result<String, SynError> {
        self.be()?;
        if !self.left().is_empty() {
            return Err(SynError::Unexpected);
        }
        Ok(self.output.to_string())
    }

    pub fn execute(&mut self, pgm: &mparse::MProgram<MInstr>) {
        let mut ic: usize;
        self.cll(0);
        match &pgm.instrs[0] {
            MInstr::ADR(_, start) => ic = *start,
            _ => panic!("invalid program prolog"),
        }
        loop {
            match &pgm.instrs[ic] {
                MInstr::Undef => panic!("Undef unexpected in program"),
                MInstr::ADR(_, _) => panic!("ADR unexpected after prolog"),
                MInstr::TST(s) => {
                    self.tst(s);
                }
                MInstr::ID => {
                    self.id();
                }
                MInstr::NUM => {
                    self.num();
                }
                MInstr::SR => {
                    self.sr();
                }
                MInstr::CLL(_, procc) => {
                    self.cll(ic + 1);
                    ic = *procc;
                    continue;
                }
                MInstr::R => {
                    ic = self.r();
                    if ic == 0 {
                        break;
                    }
                    continue;
                }
                MInstr::SET => self.set(),
                MInstr::B(_, jic) => {
                    ic = *jic;
                    continue;
                }
                MInstr::BT(_, jic) => {
                    if self.sw {
                        ic = *jic;
                        continue;
                    }
                }
                MInstr::BF(_, jic) => {
                    if !self.sw {
                        ic = *jic;
                        continue;
                    }
                }
                MInstr::BE => match self.be() {
                    Ok(Recognized) => (),
                    _ => break,
                },
                MInstr::CL(s) => self.cl(s),
                MInstr::CI => self.ci(),
                MInstr::GN1 => self.gn1(),
                MInstr::GN2 => self.gn2(),
                MInstr::LB => self.lb(),
                MInstr::OUT => self.out(),
            };
            ic += 1;
        }
    }
}

#[derive(Debug)]
pub enum MInstr {
    TST(String),
    ID,
    NUM,
    SR,
    CLL(String, usize),
    R,
    SET,
    B(String, usize),
    BT(String, usize),
    BF(String, usize),
    BE,
    CL(String),
    CI,
    GN1,
    GN2,
    LB,
    OUT,
    ADR(String, usize),
    Undef,
}

impl ParseableInstr for MInstr {
    const UNDEF: Self = MInstr::Undef;
    const ACCEPT_BLK: bool = false;

    fn is_undefined(&self) -> bool {
        matches!(self, MInstr::Undef)
    }

    fn with_label(ins: &str, label: String) -> Self {
        match ins {
            "CLL" => MInstr::CLL(label, 0),
            "B" => MInstr::B(label, 0),
            "BT" => MInstr::BT(label, 0),
            "BF" => MInstr::BF(label, 0),
            "ADR" => MInstr::ADR(label, 0),
            _ => MInstr::Undef,
        }
    }

    fn with_num(_ins: &str, _: f64) -> Self {
        MInstr::Undef
    }

    fn with_string(ins: &str, s: String) -> Self {
        match ins {
            "TST" => MInstr::TST(s),
            "CL" => MInstr::CL(s),
            _ => MInstr::Undef,
        }
    }

    fn with_noarg(ins: &str) -> Self {
        match ins {
            "ID" => MInstr::ID,
            "NUM" => MInstr::NUM,
            "SR" => MInstr::SR,
            "R" => MInstr::R,
            "SET" => MInstr::SET,
            "BE" => MInstr::BE,
            "CI" => MInstr::CI,
            "GN1" => MInstr::GN1,
            "GN2" => MInstr::GN2,
            "LB" => MInstr::LB,
            "OUT" => MInstr::OUT,
            _ => MInstr::Undef,
        }
    }

    fn aaa_of(&self) -> AAAUse {
        match self {
            MInstr::ADR(aaa, _)
            | MInstr::B(aaa, _)
            | MInstr::BT(aaa, _)
            | MInstr::BF(aaa, _)
            | MInstr::CLL(aaa, _) => AAAUse::IC(aaa.to_string()),
            _ => AAAUse::None,
        }
    }

    fn reconstruct_with_addr(&mut self, _aaa: String, _addr: u32) {
        panic!("internal error: unknown aaa instruction");
    }

    fn reconstruct_with_ic(&mut self, aaa: String, ic: usize) {
        *self = match self {
            MInstr::ADR(_, _) => MInstr::ADR(aaa, ic),
            MInstr::B(_, _) => MInstr::B(aaa, ic),
            MInstr::BT(_, _) => MInstr::BT(aaa, ic),
            MInstr::BF(_, _) => MInstr::BF(aaa, ic),
            MInstr::CLL(_, _) => MInstr::CLL(aaa, ic),
            _ => panic!("internal error: unknown aaa instruction"),
        };
    }
}

pub fn run(opts: Options) -> Result<(), Box<dyn Error>> {
    let p = mparse::load::<MInstr>(&opts.mpgm_path)?;
    let source = fs::read_to_string(&opts.source_path)?;
    let mut m = M::new(&source);
    m.execute(&p);
    match m.generated() {
        Ok(out) => {
            println!("{}", out);
            Ok(())
        }
        Err(_) => {
            println!("unexpected:\n{}", m.left());
            Err(From::from("compilation failed"))
        }
    }
}

pub struct Options {
    pub mpgm_path: String,
    pub source_path: String,
}

impl Options {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Self, &'static str> {
        args.next();
        let mpgm_path = match args.next() {
            Some(arg) => arg,
            None => return Err("missing meta machine program path argument"),
        };
        let source_path = match args.next() {
            Some(arg) => arg,
            None => return Err("missing source file path argument"),
        };
        Ok(Options {
            mpgm_path,
            source_path,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn m() {
        let m = M::new("  abc ");
        assert_eq!(m.input, "  abc ");
        assert_eq!(m.sw, false);
        assert_eq!(m.last, "");
        assert_eq!(m.output, " ".repeat(8));
    }

    #[test]
    fn m_tst() {
        let mut m = M::new("  abc_");
        assert_eq!(m.input, "  abc_");
        assert!(m.tst("abc"));
        assert_eq!(m.sw, true);
        assert_eq!(m.last, "abc");
        assert_eq!(&m.input[m.pos..], "_");
        assert!(!m.tst("__"));
        assert_eq!(m.sw, false);
        assert_eq!(m.last, "abc");
        assert_eq!(&m.input[m.pos..], "_");
    }

    #[test]
    fn m_id() {
        let mut m = M::new("  ab3c_");
        assert_eq!(m.input, "  ab3c_");
        assert!(m.id());
        assert_eq!(m.sw, true);
        assert_eq!(m.last, "ab3c");
        assert_eq!(&m.input[m.pos..], "_");
        assert!(!m.id());
        assert_eq!(m.sw, false);
        assert_eq!(m.last, "ab3c");
        assert_eq!(&m.input[m.pos..], "_");
    }

    #[test]
    fn m_num() {
        let mut m = M::new("  00.120_");
        assert_eq!(m.input, "  00.120_");
        assert!(m.num());
        assert_eq!(m.sw, true);
        assert_eq!(m.last, "00.120");
        assert_eq!(&m.input[m.pos..], "_");
        assert!(!m.num());
        assert_eq!(m.sw, false);
        assert_eq!(m.last, "00.120");
        assert_eq!(&m.input[m.pos..], "_");
    }

    #[test]
    fn m_num_not_accepted() {
        let mut m = M::new("  1.");
        assert!(!m.num());
        assert_eq!(m.sw, false);

        let mut m = M::new("  12..33");
        assert!(!m.num());
        assert_eq!(m.sw, false);
        assert_eq!(&m.input[m.pos..], "12..33")
    }

    #[test]
    fn m_sr() {
        let mut m = M::new("  'ab c  '_");
        assert_eq!(m.input, "  'ab c  '_");
        assert!(m.sr());
        assert_eq!(m.sw, true);
        assert_eq!(m.last, "'ab c  '");
        assert_eq!(&m.input[m.pos..], "_");
        assert!(!m.sr());
        assert_eq!(m.sw, false);
        assert_eq!(m.last, "'ab c  '");
        assert_eq!(&m.input[m.pos..], "_");
    }

    #[test]
    fn m_sr_unterminated() {
        let mut m = M::new("  'ab c  _");
        assert!(!m.sr());
        assert_eq!(m.sw, false);
        assert_eq!(m.last, "");
        assert_eq!(&m.input[m.pos..], "'ab c  _");
    }

    #[test]
    fn m_cll_and_gnx() {
        let mut m = M::new("");
        m.output.truncate(0);
        m.cll(100);
        m.gn1();
        m.gn2();
        m.gn2();
        m.gn1();
        let ric = m.r();
        assert_eq!(ric, 100);
        assert_eq!(m.output.as_str(), "A001 B001 B001 A001 ")
    }

    #[test]
    fn m_cll_and_gnx_nested_shallow() {
        let mut m = M::new("");
        m.output.truncate(0);
        m.cll(100);
        m.cll(200);
        m.gn1();
        m.gn2();
        m.gn1();
        let ric = m.r();
        assert_eq!(ric, 200);
        m.gn1();
        let ric = m.r();
        assert_eq!(ric, 100);
        assert_eq!(m.output.as_str(), "A001 B001 A001 A002 ")
    }

    #[test]
    fn m_cll_and_gnx_nested() {
        let mut m = M::new("");
        m.output.truncate(0);
        m.cll(100);
        m.gn1();
        m.cll(200);
        m.gn2();
        m.gn1();
        m.gn2();
        let ric = m.r();
        assert_eq!(ric, 200);
        m.gn1();
        let ric = m.r();
        assert_eq!(ric, 100);
        assert_eq!(m.output.as_str(), "A001 B001 A002 B001 A001 ")
    }

    #[test]
    fn m_switch_and_set() {
        let mut m = M::new("");
        assert_eq!(m.sw, false);
        m.set();
        assert_eq!(m.sw, true);
    }

    #[test]
    fn m_cl() {
        let mut m = M::new("");
        m.cl("ABC");
        m.cl("DEF");
        assert_eq!(m.output.as_str(), "        ABC DEF ")
    }

    #[test]
    fn m_ci() {
        let mut m = M::new("SET XYZ FOO");
        m.tst("SET");
        m.ci();
        m.id();
        m.ci();
        m.tst("END");
        m.ci();
        assert_eq!(m.output.as_str(), "        SETXYZ")
    }

    #[test]
    fn m_out_lb() {
        let mut m = M::new("");
        m.lb();
        m.cl("ABC");
        m.out();
        m.cl("DEF");
        m.out();
        m.lb();
        m.cl("XXX");
        m.out();
        assert_eq!(
            m.output.as_str(),
            r#"ABC 
        DEF 
XXX 
        "#
        )
    }
}
