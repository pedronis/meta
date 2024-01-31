use minilexer::Lexer;
use minilexer::Token;
use std::collections::HashMap;
use std::convert::From;
use std::error::Error;
use std::fs;

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

    pub fn execute(&mut self, pgm: &MProgram) {
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

type Labels = HashMap<String, u32>;
type ICs = HashMap<u32, usize>;

impl MInstr {
    fn resolve_aaa(&mut self, labels: &Labels) -> Result<(), Box<dyn Error>> {
        let aaa = match self {
            MInstr::ST(aaa, _) | MInstr::LD(aaa, _) => aaa.to_string(),
            _ => {
                return Err(From::from("internal error: unknown aaa instruction"));
            }
        };
        let addr = if let Some(addr) = labels.get(&aaa) {
            *addr
        } else {
            return Err(From::from(format!("unknown label {aaa}")));
        };
        *self = match self {
            MInstr::ST(_, _) => MInstr::ST(aaa, addr),
            MInstr::LD(_, _) => MInstr::LD(aaa, addr),
            _ => {
                return Err(From::from("internal error: unknown aaa instruction"));
            }
        };
        Ok(())
    }

    fn resolve_ic(
        &mut self,
        labels: &Labels,
        ic: &ICs,
        pgm_len: usize,
    ) -> Result<(), Box<dyn Error>> {
        let aaa = match self {
            MInstr::B(aaa, _) | MInstr::BFP(aaa, _) | MInstr::BTP(aaa, _) => aaa.to_string(),
            _ => {
                return Err(From::from("internal error: unknown aaa instruction"));
            }
        };
        let addr = if let Some(addr) = labels.get(&aaa) {
            *addr
        } else {
            return Err(From::from(format!("unknown label {aaa}")));
        };
        let ic = if let Some(ic) = ic.get(&addr) {
            *ic
        } else {
            return Err(From::from(format!(
                "internal error: unmatched addr {addr} for ic"
            )));
        };
        if ic >= pgm_len {
            return Err(From::from(format!(
                "instruction counter {ic} for {aaa} without instruction"
            )));
        }
        *self = match self {
            MInstr::B(_, _) => MInstr::B(aaa, ic),
            MInstr::BFP(_, _) => MInstr::BFP(aaa, ic),
            MInstr::BTP(_, _) => MInstr::BTP(aaa, ic),
            _ => {
                return Err(From::from("internal error: unknown aaa instruction"));
            }
        };
        Ok(())
    }
}

#[derive(Debug)]
pub struct MProgram {
    pub instrs: Vec<MInstr>,
    pub labels: Labels,
    pub ic: ICs,
    addr: u32,
}

impl MProgram {
    fn new() -> Self {
        MProgram {
            instrs: Vec::new(),
            labels: Labels::new(),
            ic: ICs::new(),
            addr: 0,
        }
    }

    fn parse(&mut self, pgm: &str) -> Result<(), Box<dyn Error>> {
        for mut line in pgm.lines() {
            line = line.trim_end();
            if line.is_empty() {
                continue;
            };
            let mut lx = Lexer::new(line, &["#"]);
            let tok = lx.next_token()?;
            match tok {
                Token::Id(id) => self.add_label(&id),
                Token::WS => {
                    let is_end = self.add_instr(&mut lx, line)?;
                    if is_end {
                        break;
                    }
                }
                Token::Symbol(s) if s == "#" => (),
                _ => return Err(From::from(format!("unexpected {:?}", tok))),
            }
        }

        //self.debug_ics();
        self.resolve()?;
        Ok(())
    }

    fn add_label(&mut self, label: &str) {
        self.labels.insert(label.to_string(), self.addr);
        self.ic.insert(self.addr, self.instrs.len());
    }

    fn add_instr(&mut self, lx: &mut Lexer, line: &str) -> Result<bool, Box<dyn Error>> {
        let ins = match lx.next_token()? {
            Token::End => return Ok(true),
            Token::Symbol(s) if s == "#" => return Ok(false),
            Token::Id(instr) => instr,
            unexp => return Err(From::from(format!("unexpected {:?}", unexp))),
        };
        let ins = ins.as_str();

        let mut tok = lx.next_token()?;
        if tok == Token::WS {
            tok = lx.next_token()?;
        }

        let mut inc = 2;
        let instr = match tok {
            Token::WS => panic!("internal error: repeated whitespace token"),
            Token::Id(label) => match ins {
                "B" => MInstr::B(label, 0),
                "ST" => MInstr::ST(label, 0),
                "LD" => MInstr::LD(label, 0),
                "BTP" => MInstr::BTP(label, 0),
                "BFP" => MInstr::BFP(label, 0),
                _ => MInstr::Undef,
            },
            Token::Num(n) => match ins {
                "BLK" => {
                    if n.fract() != 0.0 || n < 0.0 {
                        return Err(From::from("invalid BLK: {line}"));
                    }
                    self.addr += n as u32;
                    return Ok(false);
                }
                "LDL" => MInstr::LDL(n),
                _ => MInstr::Undef,
            },
            Token::Str(s) => match ins {
                "EDT" => MInstr::EDT(s),
                _ => MInstr::Undef,
            },
            Token::Symbol(s) if s != "#" => {
                return Err(From::from(format!("invalid line {line}")));
            }
            Token::End | Token::Symbol(_) => {
                inc = 1;
                match ins {
                    "EQU" => MInstr::EQU,
                    "ADD" => MInstr::ADD,
                    "SUB" => MInstr::SUB,
                    "MLT" => MInstr::MLT,
                    "PNT" => MInstr::PNT,
                    "HLT" => MInstr::HLT,
                    "END" => return Ok(true),
                    _ => MInstr::Undef,
                }
            }
        };
        if let MInstr::Undef = instr {
            return Err(From::from(format!("invalid instruction {line}")));
        }

        self.instrs.push(instr);
        self.addr += inc;
        Ok(false)
    }

    fn resolve(&mut self) -> Result<(), Box<dyn Error>> {
        let pgm_len = self.instrs.len();
        for instr in self.instrs.iter_mut() {
            match instr {
                MInstr::ST(_, _) | MInstr::LD(_, _) => instr.resolve_aaa(&self.labels)?,
                MInstr::B(_, _) | MInstr::BFP(_, _) | MInstr::BTP(_, _) => {
                    instr.resolve_ic(&self.labels, &self.ic, pgm_len)?
                }
                _ => (),
            }
        }
        Ok(())
    }

    pub fn debug_ics(&self) {
        for (label, addr) in self.labels.iter() {
            let ic = self.ic.get(addr).unwrap();
            let instr = match self.instrs.get(*ic) {
                Some(i) => i,
                None => &MInstr::Undef,
            };
            println!("{label:#?} {addr} ic:{ic} {instr:#?}")
        }
    }
}

pub fn load(pgm_path: &str) -> Result<MProgram, Box<dyn Error>> {
    let mut p = MProgram::new();
    let pgm = fs::read_to_string(pgm_path)?;
    p.parse(&pgm)?;
    Ok(p)
}

pub fn run(opts: Options) -> Result<(), Box<dyn Error>> {
    let p = load(&opts.pgm_path)?;
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
    fn mprogram_parse_vs_lexing() {
        let mut p = MProgram::new();
        assert!(p
            .parse(
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
