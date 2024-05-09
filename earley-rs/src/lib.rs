use std::{collections::HashMap, fmt::Display};

use itertools::Itertools;

use self::latex::Proof;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Token {
    Term(String),
    NonTerm(String),
}
impl Token {
    fn nonterm(&self) -> &str {
        match self {
            Token::NonTerm(s) => s,
            _ => panic!(),
        }
    }

    fn term(&self) -> &str {
        match self {
            Token::Term(s) => s,
            _ => panic!(),
        }
    }
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

impl Default for Grammar {
    fn default() -> Self {
        Self::new()
    }
}

// TODO: this could be better, let's hope that this is specific enough (spoiler probably not)
#[derive(Debug, Clone)]
enum InsertedBy {
    Pred,
    Scan(Range),
    Comp(Range, Range),
}

impl InsertedBy {
    fn scan_range(self) -> Range {
        match self {
            Self::Scan(r) => r,
            _ => panic!("{self:?} is not a scan"),
        }
    }
    fn comp_range(self) -> (Range, Range) {
        match self {
            Self::Comp(r, s) => (r, s),
            _ => panic!("{self:?} is not a comp"),
        }
    }
}

pub struct Table<I> {
    table: Vec<HashMap<Item, InsertedBy>>,
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
            .map(|x| {
                (
                    Item::init(initial.as_ref().to_owned(), x, 0..0),
                    InsertedBy::Pred,
                )
            })
            .collect();
        let input = input.into_iter();
        let table = Vec::with_capacity(input.size_hint().0);
        let mut out = Table {
            table,
            grammar,
            input,
        };
        let initials = out.pred_phase(0, initials);
        for it in initials.keys() {
            println!("inserting at 0: {it}");
        }
        println!();
        out.table.push(initials);
        out
    }

    fn scan_phase(&mut self, j: usize, token: String) -> HashMap<Item, InsertedBy> {
        let mut cur_state = HashMap::new();

        let prev_state = &self.table[j - 1];
        let token = Token::Term(token);

        for i in prev_state.keys() {
            if i.after.last() != Some(&token) {
                continue;
            }
            let range = i.range.start..i.range.end + 1;
            let mut before = i.before.clone();
            let mut after = i.after.clone();
            before.push(after.pop().unwrap());
            cur_state.insert(
                Item {
                    before,
                    after,
                    range,
                    name: i.name.clone(),
                },
                InsertedBy::Scan(i.range.clone()),
            );
        }

        cur_state
    }

    fn comp_phase_loop(
        &mut self,
        _: usize,
        cur_state: &HashMap<Item, InsertedBy>,
    ) -> Vec<(Item, InsertedBy)> {
        let mut added = vec![];
        for (item, _) in cur_state.iter() {
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
            for (candidate, _) in left_proofs.iter() {
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
                if cur_state.contains_key(&to_add) {
                    continue;
                }
                added.push((to_add, InsertedBy::Comp(left_range, right_range.to_owned())));
            }
        }

        added
    }

    fn comp_phase(
        &mut self,
        j: usize,
        mut cur_state: HashMap<Item, InsertedBy>,
    ) -> HashMap<Item, InsertedBy> {
        loop {
            let added = self.comp_phase_loop(j, &cur_state);
            if added.is_empty() {
                break;
            }
            for (add, inserted_by) in added {
                cur_state.insert(add, inserted_by);
            }
        }
        cur_state
    }

    fn pred_phase(
        &mut self,
        j: usize,
        mut cur_state: HashMap<Item, InsertedBy>,
    ) -> HashMap<Item, InsertedBy> {
        loop {
            let added = self.pred_phase_loop(j, &cur_state);
            let mut num_added = 0;
            for (add, inserted_by) in added {
                println!("predict{j}: add {add}");
                num_added += cur_state.insert(add, inserted_by).is_none() as usize;
            }

            if num_added == 0 {
                break;
            }
        }
        cur_state
    }

    fn pred_phase_loop(
        &mut self,
        j: usize,
        cur_state: &HashMap<Item, InsertedBy>,
    ) -> Vec<(Item, InsertedBy)> {
        let mut added = vec![];

        for (item, _) in cur_state.iter() {
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
                if cur_state.contains_key(&to_add) {
                    continue;
                }
                added.push((to_add.clone(), InsertedBy::Pred));
            }
        }

        added
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
        for it in cur_state.keys() {
            println!("inserting at {j}: {it}");
        }
        println!("{j}");

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
            .find(|(item, _)| item.name == initial && item.after.is_empty())
            .expect("parse had failed. if you see this you may complain about this horrid api");

        let proof = self.reconstruct_tree(self.table.len() - 1, root.0, root.1);
        latex::FullProof(proof)
    }

    fn reconstruct_tree<'a>(
        &'a self,
        j: usize,
        root: &'a Item,
        inserted_by: &InsertedBy,
    ) -> latex::Proof<'a> {
        match root.before.last() {
            None => {
                assert!(matches!(inserted_by, InsertedBy::Pred));
                Proof::Pred(root)
            }
            Some(t @ Token::Term(_)) => {
                assert!(matches!(inserted_by, InsertedBy::Scan(..)));
                let mu_range = inserted_by.clone().scan_range();
                let mu = &root.before[..root.before.len().saturating_sub(1)];
                let child = self.table[j - 1]
                    .iter()
                    .find(|(x, _)| {
                        x.after.last() == Some(t) && mu == x.before && x.range == mu_range
                    })
                    .expect("scan child exists");
                let child_proof = self.reconstruct_tree(j - 1, child.0, child.1);
                Proof::Scan(root, Box::new(child_proof))
            }
            Some(Token::NonTerm(t)) => {
                assert!(matches!(inserted_by, InsertedBy::Comp(..)));
                let (range_mu, range_b) = inserted_by.clone().comp_range();
                let child_b = self.table[j]
                    .iter()
                    .find(|(x, _)| &x.name == t && x.after.is_empty() && x.range == range_b)
                    .expect("child B must end at the same point");
                let index_mu = child_b.0.range.start;
                let name_of_b = Token::NonTerm(child_b.0.name.clone());
                let child_mu = &self.table[index_mu]
                    .iter()
                    .find(|(x, _)| {
                        x.name == root.name
                            && x.after.last() == Some(&name_of_b)
                            && x.range == range_mu
                    })
                    .expect("there's a mu item before somewhere");
                let proof_b = self.reconstruct_tree(j, child_b.0, child_b.1);
                let proof_mu = self.reconstruct_tree(index_mu, child_mu.0, child_mu.1);
                Proof::Comp(root, Box::new(proof_mu), Box::new(proof_b))
            }
        }
    }
}

pub mod latex {
    use std::fmt::Display;

    use itertools::Itertools;

    use super::{Item, Token};

    #[derive(Debug)]
    pub(super) enum Proof<'a> {
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

    pub enum ParseTree<'a> {
        Terminal(&'a str),
        NonTerminal(&'a str, Vec<ParseTree<'a>>),
    }
    pub struct FullParseTree<'a>(pub(super) ParseTree<'a>);

    impl<'a> ParseTree<'a> {
        fn from_proof(proof: Proof<'a>, rule: &'a str, mut sub: Vec<ParseTree<'a>>) -> Self {
            match proof {
                Proof::Pred(_) => todo!("not sure if i can do anything here"),
                Proof::Comp(item, mu, b) => {
                    let b_name = item.before.last().unwrap().nonterm();

                    let parse_tree_b = ParseTree::from_proof(*b, b_name, Vec::new());
                    sub.push(parse_tree_b);
                    if item.before.len() == 1 {
                        sub.reverse();
                        ParseTree::NonTerminal(rule, sub)
                    } else {
                        ParseTree::from_proof(*mu, rule, sub)
                    }
                }
                Proof::Scan(item, mu) => {
                    let a_name = item.before.last().unwrap().term();
                    sub.push(ParseTree::Terminal(a_name));
                    if item.before.len() == 1 {
                        sub.reverse();
                        ParseTree::NonTerminal(rule, sub)
                    } else {
                        ParseTree::from_proof(*mu, rule, sub)
                    }
                }
            }
        }
    }

    impl<'a> From<Proof<'a>> for ParseTree<'a> {
        fn from(value: Proof<'a>) -> Self {
            match value {
                Proof::Comp(item, _, _) | Proof::Pred(item) | Proof::Scan(item, _) => {
                    Self::from_proof(value, &item.name, Vec::new())
                }
            }
        }
    }

    impl<'a> From<FullProof<'a>> for FullParseTree<'a> {
        fn from(value: FullProof<'a>) -> Self {
            FullParseTree(value.0.into())
        }
    }

    impl Display for ParseTree<'_> {
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

    impl Display for FullParseTree<'_> {
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
}
