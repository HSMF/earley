use std::{fs::File, process::Command};

use v2::{nt, t, Grammar, Table};

mod v1;
mod v2;

fn _main() {
    use v1::*;
    let grammar = Grammar::new(
        [
            ("S".to_owned(), vec![vec![t("a"), nt("A")]]),
            (
                "A".to_owned(),
                vec![vec![t("a"), nt("A"), nt("B")], vec![t("b")]],
            ),
            ("B".to_owned(), vec![vec![t("b")]]),
        ]
        .into_iter()
        .collect(),
    );

    let input = "aaabbb".chars().map(|x| x.to_string()).collect::<Vec<_>>();
    let mut table = Table::new(grammar);

    let mut should_die = false;
    while !should_die {
        should_die = table.next(&input);
    }
}
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

    Command::new("pdflatex")
        .args(["-output-directory=target", "target/output.tex"])
        .spawn()?
        .wait()?;

    Ok(())
}
