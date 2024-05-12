use std::fmt::Display;

use itertools::Itertools;

use super::{Item, Token};

#[derive(Debug)]
pub(super) enum Proof<'a, T> {
    Comp(&'a Item<T>, Box<Proof<'a, T>>, Box<Proof<'a, T>>),
    Pred(&'a Item<T>),
    Scan(&'a Item<T>, Box<Proof<'a, T>>),
}

struct LatexProd<'a, T>(&'a str, &'a [Token<T>], &'a [Token<T>]);
struct LatexItem<'a, T>(&'a Item<T>);

impl<T> Display for LatexProd<'_, T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r"{} \ensuremath{{\to}} {} \ensuremath{{\bullet}} {}",
            self.0,
            self.1.iter().format(" "),
            self.2.iter().rev().format(" ")
        )
    }
}

impl<T> Display for LatexItem<'_, T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r"[{}, {}, {}]",
            self.0.range().start,
            self.0.range().end,
            LatexProd(self.0.name(), self.0.before(), self.0.after())
        )
    }
}

impl<T> Display for Proof<'_, T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Proof::Comp(item, mu, b) => {
                writeln!(
                    f,
                    r"\bininf{{ {item} }} {{\comp}} {{ ",
                    item = LatexItem(item)
                )?;
                writeln!(f, "{mu}")?;
                writeln!(f, r"}} {{")?;
                writeln!(f, "{b}")?;
                writeln!(f, r"}}")?;
            }
            Proof::Pred(item) => {
                writeln!(f, r"\uninf{{ {} }} {{ \pred }} {{ ", LatexItem(item))?;
                writeln!(
                    f,
                    r"\axiominf{{ {} }}{{ }}",
                    LatexProd(item.name(), item.before(), item.after()),
                )?;
                writeln!(f, r"}}")?;
            }
            Proof::Scan(item, mu) => {
                writeln!(
                    f,
                    r"\bininf{{ {item} }} {{\comp}} {{ ",
                    item = LatexItem(item)
                )?;
                writeln!(f, "{mu}")?;
                writeln!(f, r"}} {{")?;
                writeln!(
                    f,
                    r" \axiominf{{ [{}, {}, {}] }} {{ }} ",
                    item.range().end - 1,
                    item.range().end,
                    item.before().last().unwrap()
                )?;
                writeln!(f, r"}}")?;
            }
        }
        Ok(())
    }
}

pub struct FullProof<'a, T>(pub(super) Proof<'a, T>);
impl<T: Display> Display for FullProof<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            r#"
                \begin{{inctext}}[left border=20pt, right border=20pt,top border=30pt, bottom border=30pt]
                \begin{{bprooftree}}
                {body}
                \end{{bprooftree}}
                \end{{inctext}}
                "#,
            body = self.0
        )
    }
}

pub enum ParseTree<'a, T> {
    Terminal(&'a T),
    NonTerminal(&'a str, Vec<ParseTree<'a, T>>),
}
pub struct FullParseTree<'a, T>(pub(super) ParseTree<'a, T>);

impl<'a, T> ParseTree<'a, T> {
    fn from_proof(proof: Proof<'a, T>, rule: &'a str, mut sub: Vec<ParseTree<'a, T>>) -> Self {
        match proof {
            Proof::Pred(_) => todo!("not sure if i can do anything here"),
            Proof::Comp(item, mu, b) => {
                let b_name = item.before().last().unwrap().nonterm();

                let parse_tree_b = ParseTree::from_proof(*b, b_name, Vec::new());
                sub.push(parse_tree_b);
                if item.before().len() == 1 {
                    sub.reverse();
                    ParseTree::NonTerminal(rule, sub)
                } else {
                    ParseTree::from_proof(*mu, rule, sub)
                }
            }
            Proof::Scan(item, mu) => {
                let a_name = item.before().last().unwrap().term();
                sub.push(ParseTree::Terminal(a_name));
                if item.before().len() == 1 {
                    sub.reverse();
                    ParseTree::NonTerminal(rule, sub)
                } else {
                    ParseTree::from_proof(*mu, rule, sub)
                }
            }
        }
    }
}

impl<'a, T> From<Proof<'a, T>> for ParseTree<'a, T> {
    fn from(value: Proof<'a, T>) -> Self {
        match value {
            Proof::Comp(item, _, _) | Proof::Pred(item) | Proof::Scan(item, _) => {
                Self::from_proof(value, &item.name(), Vec::new())
            }
        }
    }
}

impl<'a, T> From<FullProof<'a, T>> for FullParseTree<'a, T> {
    fn from(value: FullProof<'a, T>) -> Self {
        FullParseTree(value.0.into())
    }
}

impl<T: Display> Display for ParseTree<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseTree::Terminal(s) => write!(f, "[{{{s}}}]"),
            ParseTree::NonTerminal(rule, children) => {
                write!(f, "[{rule}  {}]", children.iter().format(""))?;
                Ok(())
            }
        }
    }
}

impl<T: Display> Display for FullParseTree<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            r#"
                \begin{{inctext}}[left border=20pt, right border=20pt,top border=30pt, bottom border=30pt]
                \begin{{forest}} {body} \end{{forest}}

                \end{{inctext}}
                "#,
            body = self.0
        )
    }
}

pub struct Grammar<'a, T>(pub(super) &'a super::Grammar<T>);
impl<T: Display> Display for Grammar<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            r#"
                \begin{{inctext}}[left border=20pt, right border=20pt,top border=30pt, bottom border=30pt]
                "#,
        )?;
        // writeln!(f, r"\section*{{Grammar}}")?;
        writeln!(f, r"\begin{{lstlisting}}")?;

        for (rule, productions) in &self.0.productions {
            for production in productions {
                writeln!(f, r"{rule} -> {}", production.iter().format(" "))?;
            }
        }

        writeln!(f, r"\end{{lstlisting}}")?;
        writeln!(
            f,
            r#"
                \end{{inctext}}"#
        )?;

        Ok(())
    }
}

