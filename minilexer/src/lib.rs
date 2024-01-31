use std::convert::From;
use std::error::Error;
use std::iter::Peekable;
use std::str::Chars;

pub struct Lexer<'a> {
    it: Peekable<Chars<'a>>,
    syms: &'a [&'static str],
}

#[derive(Debug, PartialEq)]
pub enum Token {
    WS,
    Id(String),
    Num(f64),
    Symbol(String),
    Str(String),
    End,
}

use Token::*;

enum ParseState {
    ParseStart,
    ParseId,
    ParseNum,
    ParseWS,
    ParseSymbol,
    ParseStr,
}

use ParseState::*;

impl<'a> Lexer<'a> {
    pub fn new(txt: &'a str, syms: &'a [&'static str]) -> Self {
        Lexer {
            it: txt.chars().peekable(),
            syms,
        }
    }

    fn is_sym_start(&self, s: &str) -> bool {
        // XXX make a shrinking list of candidates instead?
        for symb in self.syms {
            if symb.starts_with(s) {
                return true;
            }
        }
        false
    }

    pub fn next_token(&mut self) -> Result<Token, Box<dyn Error>> {
        let mut tok = String::new();
        let mut st = ParseStart;
        while let Some(ch) = self.it.peek() {
            let ch = *ch;
            match st {
                ParseStart => {
                    if ch.is_ascii_whitespace() {
                        st = ParseWS;
                        self.it.next();
                    } else if ch.is_ascii_alphabetic() {
                        st = ParseId;
                        tok.push(ch);
                        self.it.next();
                    } else if ch.is_ascii_digit() {
                        st = ParseNum;
                        tok.push(ch);
                        self.it.next();
                    } else if ch == '\'' {
                        st = ParseStr;
                        self.it.next();
                    } else if ch == '.' {
                        tok.push(ch);
                        self.it.next();
                        st = ParseSymbol;
                        if let Some(ch) = self.it.peek() {
                            if ch.is_ascii_digit() {
                                st = ParseNum;
                                continue;
                            }
                        }
                        if !self.is_sym_start(tok.as_str()) {
                            return Err(From::from(format!("not a symbol {}", tok.as_str())));
                        }
                    } else {
                        st = ParseSymbol;
                        tok.push(ch);
                        if !self.is_sym_start(tok.as_str()) {
                            return Err(From::from(format!("not a symbol {}", tok.as_str())));
                        }
                        self.it.next();
                    }
                }
                ParseId => {
                    if ch.is_ascii_alphanumeric() {
                        tok.push(ch);
                        self.it.next();
                    } else {
                        break;
                    }
                }
                ParseNum => {
                    if ch.is_ascii_digit() || ch == '.' {
                        tok.push(ch);
                        self.it.next();
                    } else {
                        break;
                    }
                }
                ParseWS => {
                    if ch.is_ascii_whitespace() {
                        self.it.next();
                    } else {
                        break;
                    }
                }
                ParseSymbol => {
                    tok.push(ch);
                    if !self.is_sym_start(tok.as_str()) {
                        tok.pop();
                        break;
                    }
                    self.it.next();
                }
                ParseStr => {
                    self.it.next();
                    if ch == '\'' {
                        break;
                    }
                    tok.push(ch);
                }
            };
        }
        let tok = match st {
            ParseNum => {
                let n = tok.parse::<f64>()?;
                Num(n)
            }
            ParseStart => End,
            ParseWS => WS,
            ParseId => Id(tok),
            ParseSymbol => Symbol(tok),
            ParseStr => Str(tok),
        };
        Ok(tok)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        let mut lx = Lexer::new("abc", &[]);
        assert_eq!(lx.next_token().expect("token"), Id("abc".to_string()));
        assert_eq!(lx.next_token().expect("token"), End);
    }

    #[test]
    fn ws_par_ws_abc_par_etc() {
        let mut lx = Lexer::new("  ( a2bc) ( .5 2.3) .do ", &["(", ")", ".do"]);
        assert_eq!(lx.next_token().expect("token"), WS);
        assert_eq!(lx.next_token().expect("token"), Symbol("(".to_string()));
        assert_eq!(lx.next_token().expect("token"), WS); // this is not mandatory, see next test
        assert_eq!(lx.next_token().expect("token"), Id("a2bc".to_string()));
        assert_eq!(lx.next_token().expect("token"), Symbol(")".to_string()));
        assert_eq!(lx.next_token().expect("token"), WS);
        assert_eq!(lx.next_token().expect("token"), Symbol("(".to_string()));
        assert_eq!(lx.next_token().expect("token"), WS);
        assert_eq!(lx.next_token().expect("token"), Num(0.5));
        assert_eq!(lx.next_token().expect("token"), WS);
        assert_eq!(lx.next_token().expect("token"), Num(2.3));
        assert_eq!(lx.next_token().expect("token"), Symbol(")".to_string()));
        assert_eq!(lx.next_token().expect("token"), WS);
        assert_eq!(lx.next_token().expect("token"), Symbol(".do".to_string()));
        assert_eq!(lx.next_token().expect("token"), WS);
        assert_eq!(lx.next_token().expect("token"), End);
    }

    #[test]
    fn ws_par_ws_abc_par_etc_ws_not_secessary() {
        let mut lx = Lexer::new("  (abc) (.5 2.3) .do", &["(", ")", ".do"]);
        assert_eq!(lx.next_token().expect("token"), WS);
        assert_eq!(lx.next_token().expect("token"), Symbol("(".to_string()));
        assert_eq!(lx.next_token().expect("token"), Id("abc".to_string()));
        assert_eq!(lx.next_token().expect("token"), Symbol(")".to_string()));
        assert_eq!(lx.next_token().expect("token"), WS);
        assert_eq!(lx.next_token().expect("token"), Symbol("(".to_string()));
        assert_eq!(lx.next_token().expect("token"), Num(0.5));
        assert_eq!(lx.next_token().expect("token"), WS);
        assert_eq!(lx.next_token().expect("token"), Num(2.3));
        assert_eq!(lx.next_token().expect("token"), Symbol(")".to_string()));
        assert_eq!(lx.next_token().expect("token"), WS);
        assert_eq!(lx.next_token().expect("token"), Symbol(".do".to_string()));
        assert_eq!(lx.next_token().expect("token"), End);
    }

    #[test]
    fn ws_str_str_ws_str_etc() {
        let mut lx = Lexer::new("  'a b''c d,  e' 'fz' ''", &[]);
        assert_eq!(lx.next_token().expect("token"), WS);
        assert_eq!(lx.next_token().expect("token"), Str("a b".to_string()));
        assert_eq!(lx.next_token().expect("token"), Str("c d,  e".to_string()));
        assert_eq!(lx.next_token().expect("token"), WS);
        assert_eq!(lx.next_token().expect("token"), Str("fz".to_string()));
        assert_eq!(lx.next_token().expect("token"), WS);
        assert_eq!(lx.next_token().expect("token"), Str("".to_string()));
    }

    #[test]
    fn num_error() {
        let mut lx = Lexer::new("1.2.3", &[]);
        if let Err(e) = lx.next_token() {
            assert_eq!(format!("{}", e), "invalid float literal")
        } else {
            assert!(false)
        }
    }

    #[test]
    fn symbol_error_dot() {
        let mut lx = Lexer::new(".do", &["(", ")"]);
        if let Err(e) = lx.next_token() {
            assert_eq!(format!("{}", e), "not a symbol .")
        } else {
            assert!(false)
        }
    }

    #[test]
    fn symbol_error_other() {
        let mut lx = Lexer::new("-", &["+"]);
        if let Err(e) = lx.next_token() {
            assert_eq!(format!("{}", e), "not a symbol -")
        } else {
            assert!(false)
        }
    }
}
