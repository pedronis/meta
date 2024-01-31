use minilexer::Lexer;
use minilexer::Token;
use std::collections::HashMap;
use std::convert::From;
use std::error::Error;
use std::fs;

#[derive(Debug)]
pub struct MProgram<MInstr: ParseableInstr + std::fmt::Debug> {
    pub instrs: Vec<MInstr>,
    pub labels: Labels,
    pub ic: ICs,
    addr: u32,
}

type Labels = HashMap<String, u32>;
type ICs = HashMap<u32, usize>;

pub enum AAAUse {
    Mem(String),
    IC(String),
    None,
}

pub trait ParseableInstr {
    const UNDEF: Self;
    const ACCEPT_BLK: bool;

    fn is_undefined(&self) -> bool;

    fn with_label(ins: &str, label: String) -> Self;
    fn with_num(ins: &str, n: f64) -> Self;
    fn with_string(ins: &str, s: String) -> Self;
    fn with_noarg(ins: &str) -> Self;

    fn aaa_of(&self) -> AAAUse;
    fn reconstruct_with_addr(&mut self, aaa: String, addr: u32);
    fn reconstruct_with_ic(&mut self, aaa: String, ic: usize);
}

fn resolve_aaa<MInstr: ParseableInstr>(
    instr: &mut MInstr,
    aaa: String,
    labels: &Labels,
) -> Result<(), Box<dyn Error>> {
    let addr = if let Some(addr) = labels.get(&aaa) {
        *addr
    } else {
        return Err(From::from(format!("unknown label {aaa}")));
    };
    instr.reconstruct_with_addr(aaa, addr);
    Ok(())
}

fn resolve_ic<MInstr: ParseableInstr>(
    instr: &mut MInstr,
    aaa: String,
    labels: &Labels,
    ic: &ICs,
    pgm_len: usize,
) -> Result<(), Box<dyn Error>> {
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
    instr.reconstruct_with_ic(aaa, ic);
    Ok(())
}

impl<MInstr: ParseableInstr + std::fmt::Debug> MProgram<MInstr> {
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
            Token::Id(label) => MInstr::with_label(ins, label),
            Token::Num(n) => {
                if ins == "BLK" {
                    if !MInstr::ACCEPT_BLK {
                        return Err(From::from("BLK use is invalid"));
                    }
                    if n.fract() != 0.0 || n < 0.0 {
                        return Err(From::from("invalid BLK: {line}"));
                    }
                    self.addr += n as u32;
                    return Ok(false);
                }
                MInstr::with_num(ins, n)
            }
            Token::Str(s) => MInstr::with_string(ins, s),
            Token::Symbol(s) if s != "#" => {
                return Err(From::from(format!("invalid line {line}")));
            }
            Token::End | Token::Symbol(_) => {
                inc = 1;
                if ins == "END" {
                    return Ok(true);
                }
                MInstr::with_noarg(ins)
            }
        };
        if instr.is_undefined() {
            return Err(From::from(format!("invalid instruction {line}")));
        }

        self.instrs.push(instr);
        self.addr += inc;
        Ok(false)
    }

    fn resolve(&mut self) -> Result<(), Box<dyn Error>> {
        let pgm_len = self.instrs.len();
        for instr in self.instrs.iter_mut() {
            match instr.aaa_of() {
                AAAUse::Mem(aaa) => resolve_aaa(instr, aaa, &self.labels)?,
                AAAUse::IC(aaa) => resolve_ic(instr, aaa, &self.labels, &self.ic, pgm_len)?,
                AAAUse::None => (),
            }
        }
        Ok(())
    }

    pub fn debug_ics(&self) {
        for (label, addr) in self.labels.iter() {
            let ic = self.ic.get(addr).unwrap();
            let mut instr = &MInstr::UNDEF;
            instr = match self.instrs.get(*ic) {
                Some(i) => i,
                None => instr,
            };
            println!("{label:#?} {addr} ic:{ic} {instr:#?}")
        }
    }
}

pub fn load<MInstr: ParseableInstr + std::fmt::Debug>(
    pgm_path: &str,
) -> Result<MProgram<MInstr>, Box<dyn Error>> {
    let mut p = MProgram::new();
    let pgm = fs::read_to_string(pgm_path)?;
    p.parse(&pgm)?;
    Ok(p)
}

pub fn parse<MInstr: ParseableInstr + std::fmt::Debug>(
    pgm: &str,
) -> Result<MProgram<MInstr>, Box<dyn Error>> {
    let mut p = MProgram::new();
    p.parse(pgm)?;
    Ok(p)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    enum MInstr {
        B(String, usize),
        LDL(f64),
        ST(String, u32),
        LD(String, u32),
        EDT(String),
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
                "HLT" => MInstr::HLT,
                _ => MInstr::Undef,
            }
        }

        fn aaa_of(&self) -> AAAUse {
            match self {
                MInstr::ST(aaa, _) | MInstr::LD(aaa, _) => AAAUse::Mem(aaa.to_string()),
                MInstr::B(aaa, _) => AAAUse::IC(aaa.to_string()),
                _ => AAAUse::None,
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
                _ => panic!("internal error: unknown aaa instruction"),
            };
        }
    }

    #[test]
    fn it_works() {
        assert!(parse::<MInstr>(
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
   HLT
   EDT'233'
   END#comment
"#
        )
        .is_ok())
    }
}
