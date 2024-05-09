use std::{fs::File, process::Command};

use earley::{latex::FullParseTree, nt, t, Grammar, Table};

fn main() -> anyhow::Result<()> {
    let mut grammar = Grammar::new();

    grammar.add_prod("S", [nt("M")]);
    grammar.add_prod("M", [nt("M"), t("+"), nt("M")]);
    grammar.add_prod("M", [nt("num")]);

    grammar.add_prod("num", [nt("digit")]);
    grammar.add_prod("num", [nt("digit"), nt("num")]);
    grammar.add_prod("digit", [t("1")]);
    grammar.add_prod("digit", [t("2")]);
    grammar.add_prod("digit", [t("3")]);
    grammar.add_prod("digit", [t("4")]);
    grammar.add_prod("digit", [t("5")]);
    grammar.add_prod("digit", [t("6")]);
    grammar.add_prod("digit", [t("7")]);
    grammar.add_prod("digit", [t("8")]);
    grammar.add_prod("digit", [t("9")]);
    grammar.add_prod("digit", [t("0")]);

    let mut table = Table::new(grammar, "S", "21+29+73".chars().map(|x| x.to_string()));
    table.parse();

    let proof = table.reconstruct("S");
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
