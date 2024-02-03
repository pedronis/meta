use std::env;
use std::fs;

fn main() {
    let mut args = env::args();
    args.next();
    let syn_path = args.next().expect("missing syntax file path");
    let syntax = fs::read_to_string(syn_path).expect("cannot read syntax file");
    let mut m = meta::M::new(&syntax);
    let _ = program(&mut m);
    match m.generated() {
        Ok(out) => println!("{}", out),
        Err(_) => println!("unexpected:\n{}", m.left()),
    }
}

fn with_cll<F>(lvl: usize, m: &mut meta::M, recog: F) -> meta::MResult
where
    F: Fn(&mut meta::M) -> meta::MResult,
{
    m.cll(lvl);
    let res = recog(m);
    if m.r() != lvl {
        panic!("internal recursion stack error")
    }
    res
}

fn out1(m: &mut meta::M) -> meta::MResult {
    with_cll(5, m, |m| {
        if m.tst("*1") {
            m.cl("GN1");
        } else if m.tst("*2") {
            m.cl("GN2");
        } else if m.tst("*") {
            m.cl("CI");
        } else if m.sr() {
            m.cl("CL ");
            m.ci();
        } else {
            return Ok(meta::Unrecognized);
        }
        m.out();
        Ok(meta::Recognized)
    })
}

fn output(m: &mut meta::M) -> meta::MResult {
    with_cll(4, m, |m| {
        if m.tst(".OUT") {
            m.tst("(");
            m.be()?;
            while let meta::Recognized = out1(m)? {}
            m.tst(")");
            m.be()?;
        } else if m.tst(".LABEL") {
            m.cl("LB");
            m.out();
            if let meta::Unrecognized = out1(m)? {
                return Err(meta::SynError::Unexpected);
            }
        } else {
            return Ok(meta::Unrecognized);
        }
        m.cl("OUT");
        m.out();
        Ok(meta::Recognized)
    })
}

fn ex3(m: &mut meta::M) -> meta::MResult {
    with_cll(3, m, |m| {
        if m.id() {
            m.cl("CLL");
            m.ci();
            m.out();
        } else if m.sr() {
            m.cl("TST ");
            m.ci();
            m.out();
        } else if m.tst(".ID") {
            m.cl("ID");
            m.out();
        } else if m.tst(".NUMBER") {
            m.cl("NUM");
            m.out();
        } else if m.tst(".STRING") {
            m.cl("SR");
            m.out();
        } else if m.tst("(") {
            if let meta::Unrecognized = ex1(m)? {
                return Err(meta::SynError::Unexpected);
            }
            m.tst(")");
            m.be()?;
        } else if m.tst(".EMPTY") {
            m.cl("SET");
            m.out();
        } else if m.tst("$") {
            m.lb();
            m.gn1();
            m.out();
            if let meta::Unrecognized = ex3(m)? {
                return Err(meta::SynError::Unexpected);
            }
            m.cl("BT ");
            m.gn1();
            m.out();
            m.cl("SET");
            m.out();
        } else {
            return Ok(meta::Unrecognized);
        }
        Ok(meta::Recognized)
    })
}

fn ex2(m: &mut meta::M) -> meta::MResult {
    with_cll(2, m, |m| {
        if let meta::Recognized = ex3(m)? {
            m.cl("BF ");
            m.gn1();
            m.out();
        } else if let meta::Unrecognized = output(m)? {
            return Ok(meta::Unrecognized);
        }
        loop {
            if let meta::Recognized = ex3(m)? {
                m.cl("BE");
                m.out();
            } else if let meta::Unrecognized = output(m)? {
                break;
            }
        }
        // set
        m.lb();
        m.gn1();
        m.out();
        Ok(meta::Recognized)
    })
}

fn ex1(m: &mut meta::M) -> meta::MResult {
    m.cll(1);
    if let meta::Unrecognized = ex2(m)? {
        return Ok(meta::Unrecognized);
    }
    loop {
        if !m.tst("/") {
            break;
        }
        m.cl("BT ");
        m.gn1();
        m.out();
        if let meta::Unrecognized = ex2(m)? {
            return Err(meta::SynError::Unexpected);
        }
    }
    // set
    m.lb();
    m.gn1();
    m.out();
    let rc = m.r();
    if rc != 1 {
        panic!("internal recursion stack error")
    }
    Ok(meta::Recognized)
}

fn st(m: &mut meta::M) -> meta::MResult {
    if !m.id() {
        return Ok(meta::Unrecognized);
    }
    m.lb();
    m.ci();
    m.out();
    m.tst("=");
    m.be()?;
    if let meta::Unrecognized = ex1(m)? {
        return Err(meta::SynError::Unexpected);
    }
    m.tst(";");
    m.be()?;
    m.cl("R");
    m.out();
    Ok(meta::Recognized)
}

fn program(m: &mut meta::M) -> meta::MResult {
    if !m.tst(".SYNTAX") {
        return Ok(meta::Unrecognized);
    }
    m.id();
    m.be()?;
    m.cl("ADR");
    m.ci();
    m.out();
    while let meta::Recognized = st(m)? {}
    // set
    m.tst(".END");
    m.be()?;
    m.cl("END");
    m.out();
    Ok(meta::Recognized)
}

#[test]
fn program_works() {
    let mut m = meta::M::new(
        r#"
.SYNTAX A

A =  X / 'Y' ;

.END
"#,
    );
    assert!(matches!(program(&mut m), Ok(meta::Recognized)));
    assert_eq!(
        m.generated().expect("failed regcognition"),
        r#"        ADR A
A
        CLL X
        BF  A001 
A001 
        BT  A002 
        TST  'Y'
        BF  A003 
A003 
A002 
        R 
        END 
        "#
    )
}
