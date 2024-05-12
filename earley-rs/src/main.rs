use std::{fs::File, process::Command};

use earley::{latex::FullParseTree, Grammar, Parser, PrefixParser, Token};

fn tmp() -> anyhow::Result<()> {
    fn t(c: char) -> Token<char> {
        Token::Term(c)
    }
    fn nt(c: impl ToString) -> Token<char> {
        Token::NonTerm(c.to_string())
    }

    const S: &str = "S";
    let mut grammar = Grammar::new();
    grammar.add_prod(S, [nt(S), nt(S)]);
    grammar.add_prod(S, [t('('), nt(S), t(')')]);
    grammar.add_prod(S, [t('('), t(')')]);

    let mut parser = PrefixParser::new(grammar, "S");

    let mut parsed = String::new();
    for line in std::io::stdin().lines() {
        let line = line?;
        let ch = line.chars().next().unwrap();
        if let Ok(()) = parser.try_next(ch) {
            parsed.push(ch);
            println!("parse success");
        } else {
            println!("parse error")
        }

        println!("currently parsing {parsed:?}");
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    tmp()?;
    panic!();
    let mut grammar = Grammar::new();

    fn t(c: char) -> Token<char> {
        Token::Term(c)
    }
    fn nt(c: impl ToString) -> Token<char> {
        Token::NonTerm(c.to_string())
    }

    grammar.add_prod("S", [nt("M")]);
    grammar.add_prod("M", [nt("M"), t('+'), nt("M")]);
    grammar.add_prod("M", [nt("num")]);

    grammar.add_prod("num", [nt("digit")]);
    grammar.add_prod("num", [nt("digit"), nt("num")]);
    grammar.add_prod("digit", [t('1')]);
    grammar.add_prod("digit", [t('2')]);
    grammar.add_prod("digit", [t('3')]);
    grammar.add_prod("digit", [t('4')]);
    grammar.add_prod("digit", [t('5')]);
    grammar.add_prod("digit", [t('6')]);
    grammar.add_prod("digit", [t('7')]);
    grammar.add_prod("digit", [t('8')]);
    grammar.add_prod("digit", [t('9')]);
    grammar.add_prod("digit", [t('0')]);

    let parser = Parser::new("((((())))))".chars(), grammar.to_owned(), "S");
    let parse_result = parser.parse()?;

    let proof = parse_result.reconstruct();
    use std::io::Write;

    let mut f = File::create("target/output.tex")?;
    writeln!(
        f,
        r#"
                \documentclass{{article}}
                \usepackage{{commands}}
                \usepackage{{forest}}
                \begin{{document}}
    "#
    )?;
    write!(f, "{}", grammar.latex())?;
    write!(f, "{proof}")?;
    let tree = FullParseTree::from(proof);
    write!(f, "{tree}")?;
    writeln!(
        f,
        r#"
                \end{{document}}
    "#
    )?;

    eprintln!("running pdflatex to compile parse tree");
    Command::new("pdflatex")
        .args(["-output-directory=target", "target/output.tex"])
        // .stdout(Stdio::null())
        .spawn()?
        .wait()?;
    eprintln!("there should now be a pdf at ./target/output.pdf");
    Ok(())
}
