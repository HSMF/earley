use std::{collections::HashMap, fmt::Display};

use itertools::Itertools;

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

type Range = std::ops::Range<usize>;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Item {
    AxiomSource(Range, String),
    Prod(Range, String, Vec<Token>, Vec<Token>),
}

impl Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Item::AxiomSource(r, s) => {
                write!(f, "[{}, {}, {}]", r.start, r.end, s)?;
            }
            Item::Prod(r, name, before, after) => {
                write!(
                    f,
                    "[{}, {}, {name} -> {} . {}]",
                    r.start,
                    r.end,
                    before.iter().format(" "),
                    after.iter().format(" ")
                )?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Grammar {
    productions: HashMap<String, Vec<Vec<Token>>>,
}

impl Grammar {
    pub fn new(productions: HashMap<String, Vec<Vec<Token>>>) -> Self {
        Self { productions }
    }
}


pub struct Table {
    table: HashMap<usize, Vec<Item>>,
    grammar: Grammar,
    j: usize,
}
impl Table {
    pub fn new(grammar: Grammar) -> Self {
        let first = grammar
            .productions
            .get("S")
            .unwrap()
            .to_owned()
            .first()
            .unwrap()
            .to_owned();
        let first = Item::Prod(0..0, "S".to_owned(), vec![], first);
        Table {
            table: HashMap::from([(0, vec![first])]),
            grammar,
            j: 0,
        }
    }

    pub fn next(&mut self, input: &[String]) -> bool {
        self.j += 1;
        let j = self.j;

        // phase 1 : scan
        // use axiom j-1,j,i[j-1] to advance in state j-1
        // -> keep advanced (scan)

        let rn = input[j - 1].clone();
        let mut cur_state = vec![Item::AxiomSource(j - 1..j, input[j - 1].clone())];
        {
            let prev_state = self.table.get(&(j - 1)).unwrap();
            for i in prev_state {
                let Item::Prod(mut range, rule, mut before, mut after) = i.clone() else {
                    continue;
                };
                if after.first().is_some_and(|x| x == &Token::Term(rn.clone())) {
                    range.end += 1;
                    before.push(after[0].clone());
                    after.remove(0);
                    cur_state.push(Item::Prod(range, rule.clone(), before, after))
                }
            }
        };

        loop {
            let mut added = vec![];

            for item in &cur_state {
                let Item::Prod(range, rule, _, after) = item else {
                    continue;
                };
                assert_eq!(range.end, j);
                if !after.is_empty() {
                    continue;
                }

                let rule = Token::NonTerm(rule.clone());
                let left_proofs = self.table.get(&range.start).unwrap().clone();
                for candidate in &left_proofs {
                    let Item::Prod(left_range, other_rule, mut before, mut after) =
                        candidate.clone()
                    else {
                        continue;
                    };

                    if after.first() == Some(&rule) {
                        before.push(after[0].clone());
                        after.remove(0);
                        let to_add = Item::Prod(
                            left_range.start..range.end,
                            other_rule.clone(),
                            before,
                            after,
                        );
                        if cur_state.contains(&to_add) {
                            continue;
                        }
                        added.push(to_add)
                    }
                }
            }

            if added.is_empty() {
                break;
            }
            cur_state.extend_from_slice(&added);
        }
        // phase 2:
        // for all item completed
        //   comp with item
        // if added some
        //   restart phase 2

        // phase 3:
        // pred
        {
            let mut added = vec![];

            for item in &cur_state {
                let Item::Prod(_, _, _, after) = item else {
                    continue;
                };

                let Some(Token::NonTerm(prediction)) = after.first() else {
                    continue;
                };

                let productions = self.grammar.productions.get(prediction).unwrap();
                for i in productions {
                    let to_add = Item::Prod(j..j, prediction.clone(), vec![], i.clone());
                    if cur_state.contains(&to_add) {
                        continue;
                    }
                    added.push(to_add);
                }
            }
            cur_state.extend_from_slice(&added);
        }

        for it in &cur_state {
            println!("inserting at {j}: {it}");
        }
        println!();
        self.table.insert(j, cur_state);

        self.j == input.len()
    }
}

