use std::{
    fs::File,
    process::{Command, Stdio},
};

use earley::{nt, t, Grammar, Table};

fn main() -> anyhow::Result<()> {
    let mut grammar = Grammar::new();
    grammar.add_prod("S", [t("a"), nt("A")]);
    grammar.add_prod("A", [t("a"), nt("A"), nt("B")]);
    grammar.add_prod("A", [t("b")]);
    grammar.add_prod("B", [t("b")]);

    let mut table = Table::new(grammar, "S", "aaabbb".chars().map(|x| x.to_string()));
    table.parse();

    let proof = table.reconstruct("S");
    use std::io::Write;

    let mut f = File::create("target/output.tex")?;
    write!(f, "{proof}")?;

    eprintln!("running pdflatex to compile parse tree");
    Command::new("pdflatex")
        .args(["-output-directory=target", "target/output.tex"])
        .stdout(Stdio::null())
        .spawn()?
        .wait()?;
    eprintln!("there should now be a pdf at ./target/output.pdf");

    Ok(())
}
