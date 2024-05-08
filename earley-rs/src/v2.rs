use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use itertools::Itertools;

use self::latex::Proof;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Token {
    Term(String),
    NonTerm(String),
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Term(t) => write!(f, "`{t}`"),
            Token::NonTerm(t) => write!(f, "{t}"),
        }
    }
}

pub fn t(s: &str) -> Token {
    Token::Term(s.to_string())
}
pub fn nt(s: &str) -> Token {
    Token::NonTerm(s.to_string())
}

pub type Range = std::ops::Range<usize>;

#[derive(Clone, Hash, PartialEq, Eq)]
struct Item {
    range: Range,
    name: String,
    before: Vec<Token>,
    /// After is stored in reverse order for efficient pushing
    ///
    /// using a deque is boring
    after: Vec<Token>,
}

impl std::fmt::Debug for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Item{self}")
    }
}

impl Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Item {
            range,
            name,
            before,
            after,
        } = self;
        {
            write!(
                f,
                "[{}, {}, {name} -> {} . {}]",
                range.start,
                range.end,
                before.iter().format(" "),
                after.iter().rev().format(" ")
            )?;
        }
        Ok(())
    }
}

impl Item {
    pub fn init(name: String, mut after: Vec<Token>, range: Range) -> Self {
        after.reverse();
        Self {
            range,
            name,
            before: Vec::new(),
            after,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Grammar {
    productions: HashMap<String, Vec<Vec<Token>>>,
}

impl Grammar {
    pub fn new() -> Self {
        Self {
            productions: HashMap::new(),
        }
    }

    pub fn add_prod(&mut self, nonterm: impl ToString, expansion: impl IntoIterator<Item = Token>) {
        let nonterm = nonterm.to_string();
        let expansion = expansion.into_iter().collect();
        let entry = self.productions.entry(nonterm).or_default();
        entry.push(expansion);
    }
}

pub struct Table<I> {
    table: Vec<HashSet<Item>>,
    grammar: Grammar,
    input: I,
}

impl<I> Table<I>
where
    I: Iterator<Item = String>,
{
    pub fn new(
        grammar: Grammar,
        initial: impl AsRef<str>,
        input: impl IntoIterator<IntoIter = I>,
    ) -> Self {
        let initials = grammar
            .productions
            .get(initial.as_ref())
            .expect("grammar must contain initial")
            .iter()
            .cloned()
            .map(|x| Item::init(initial.as_ref().to_owned(), x, 0..0))
            .collect();
        let input = input.into_iter();
        let table = {
            let mut vec = Vec::with_capacity(input.size_hint().0);
            vec.push(initials);
            vec
        };
        Table {
            table,
            grammar,
            input,
        }
    }

    fn scan_phase(&mut self, j: usize, token: String) -> HashSet<Item> {
        let mut cur_state = HashSet::new();

        let prev_state = &self.table[j - 1];
        let token = Token::Term(token);

        for i in prev_state {
            if i.after.last() != Some(&token) {
                continue;
            }
            let range = i.range.start..i.range.end + 1;
            let mut before = i.before.clone();
            let mut after = i.after.clone();
            before.push(after.pop().unwrap());
            cur_state.insert(Item {
                before,
                after,
                range,
                name: i.name.clone(),
            });
        }

        cur_state
    }

    fn comp_phase_loop(&mut self, _: usize, cur_state: &HashSet<Item>) -> Vec<Item> {
        let mut added = vec![];
        for item in cur_state.iter() {
            let Item {
                range: right_range,
                name,
                after,
                ..
            } = item;
            if !after.is_empty() {
                continue;
            }
            let right_rule = Token::NonTerm(name.clone());
            let left_proofs = &self.table[right_range.start];
            for candidate in left_proofs.iter() {
                let Item {
                    range: left_range,
                    name,
                    mut before,
                    mut after,
                } = candidate.clone();
                if after.last() != Some(&right_rule) {
                    continue;
                }
                before.push(after.pop().unwrap());
                let to_add = Item {
                    range: left_range.start..right_range.end,
                    name,
                    before,
                    after,
                };
                if cur_state.contains(&to_add) {
                    continue;
                }
                added.push(to_add);
            }
        }

        added
    }

    fn comp_phase(&mut self, j: usize, mut cur_state: HashSet<Item>) -> HashSet<Item> {
        loop {
            let added = self.comp_phase_loop(j, &cur_state);
            if added.is_empty() {
                break;
            }
            for add in added {
                cur_state.insert(add);
            }
        }
        cur_state
    }

    fn pred_phase(&mut self, j: usize, mut cur_state: HashSet<Item>) -> HashSet<Item> {
        let mut added = vec![];

        for item in cur_state.iter() {
            let Some(Token::NonTerm(prediction)) = item.after.last() else {
                continue;
            };
            let productions = self
                .grammar
                .productions
                .get(prediction)
                .expect("grammar must contain nonterminal");
            for i in productions {
                let to_add = Item::init(prediction.clone(), i.clone(), j..j);
                if cur_state.contains(&to_add) {
                    continue;
                }
                added.push(to_add);
            }
        }

        for add in added {
            cur_state.insert(add);
        }

        cur_state
    }

    fn next(&mut self, token: String) {
        let j = self.table.len();

        // # phase 1 : scan
        // use axiom j-1,j,i[j-1] to advance in state j-1
        // -> keep advanced (scan)
        let cur_state = self.scan_phase(j, token);

        // # phase 2: comp
        // for all item completed
        //   comp with item
        // if added some
        //   restart phase 2
        let cur_state = self.comp_phase(j, cur_state);

        // phase 3:
        // pred
        let cur_state = self.pred_phase(j, cur_state);

        // report the table
        for it in &cur_state {
            println!("inserting at {j}: {it}");
        }
        println!();

        self.table.push(cur_state)
    }

    pub fn parse(&mut self) {
        while let Some(token) = self.input.next() {
            self.next(token)
        }
    }

    pub fn reconstruct(&self, initial: impl AsRef<str>) -> latex::FullProof {
        let initial = initial.as_ref();
        let root = self
            .table
            .last()
            .unwrap()
            .iter()
            .find(|item| item.name == initial && item.after.is_empty())
            .expect("parse had failed. if you see this you may complain about this horrid api");

        let proof = self.reconstruct_tree(self.table.len() - 1, root);
        latex::FullProof(proof)
    }

    fn reconstruct_tree<'a>(&'a self, j: usize, root: &'a Item) -> latex::Proof<'a> {
        match root.before.last() {
            None => Proof::Pred(root),
            Some(t @ Token::Term(_)) => {
                dbg!(&root, &self.table[j - 1]);
                let mu = &root.before[..root.before.len().saturating_sub(1)];
                let child = self.table[j - 1]
                    .iter()
                    .find(|x| x.after.last() == Some(t) && mu == x.before)
                    .expect("scan child exists");
                let child_proof = self.reconstruct_tree(j - 1, child);
                Proof::Scan(root, Box::new(child_proof))
            }
            Some(Token::NonTerm(t)) => {
                let child_b = self.table[j]
                    .iter()
                    .find(|x| &x.name == t && x.after.is_empty())
                    .expect("child B must end at the same point");
                let index_mu = child_b.range.start;
                let name_of_b = Token::NonTerm(child_b.name.clone());
                let child_mu = self.table[index_mu]
                    .iter()
                    .find(|x| x.name == root.name && x.after.last() == Some(&name_of_b))
                    .expect("there's a mu item before somewhere");
                let proof_b = self.reconstruct_tree(j, child_b);
                let proof_mu = self.reconstruct_tree(index_mu, child_mu);
                Proof::Comp(root, Box::new(proof_mu), Box::new(proof_b))
            }
        }
    }
}

mod latex {
    use std::fmt::Display;

    use itertools::Itertools;

    use super::{Item, Token};

    pub enum Proof<'a> {
        Comp(&'a Item, Box<Proof<'a>>, Box<Proof<'a>>),
        Pred(&'a Item),
        Scan(&'a Item, Box<Proof<'a>>),
    }

    struct LatexProd<'a>(&'a str, &'a [Token], &'a [Token]);
    struct LatexItem<'a>(&'a Item);

    impl Display for LatexProd<'_> {
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

    impl Display for LatexItem<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                r"[{}, {}, {}]",
                self.0.range.start,
                self.0.range.end,
                LatexProd(&self.0.name, &self.0.before, &self.0.after)
            )
        }
    }

    impl Display for Proof<'_> {
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
                        LatexProd(&item.name, &item.before, &item.after),
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
                        item.range.end - 1,
                        item.range.end,
                        item.before.last().unwrap()
                    )?;
                    writeln!(f, r"}}")?;
                }
            }
            Ok(())
        }
    }

    pub struct FullProof<'a>(pub(super) Proof<'a>);
    impl Display for FullProof<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            writeln!(
                f,
                r#"
                \documentclass{{article}}
                \usepackage{{commands}}
                \begin{{document}}
                \begin{{inctext}}[left border=20pt, right border=20pt,top border=30pt, bottom border=30pt]
                \begin{{bprooftree}}
                {body}
                \end{{bprooftree}}

                \end{{inctext}}
                \end{{document}}
                "#,
                body = self.0
            )
        }
    }
}
