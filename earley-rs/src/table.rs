use std::{collections::HashMap, fmt::Display};

use itertools::Itertools;

use crate::{Grammar, InsertedBy, Range, Token};

#[derive(Clone, Hash, PartialEq, Eq)]
pub(crate) struct Item<T> {
    range: Range,
    name: String,
    before: Vec<Token<T>>,
    /// After is stored in reverse order for efficient pushing
    ///
    /// using a deque is boring
    after: Vec<Token<T>>,
}

impl<T: std::fmt::Debug> std::fmt::Debug for Item<T> {
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
                "[{}, {}, {name} -> {:?} . {:?}]",
                range.start,
                range.end,
                before.iter().format(" "),
                after.iter().rev().format(" ")
            )?;
        }
        Ok(())
    }
}

impl<T: Display> Display for Item<T> {
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

impl<T> Item<T> {
    pub fn init(name: String, mut after: Vec<Token<T>>, range: Range) -> Self {
        after.reverse();
        Self {
            range,
            name,
            before: Vec::new(),
            after,
        }
    }

    pub(crate) fn before(&self) -> &[Token<T>] {
        &self.before
    }

    pub(crate) fn after(&self) -> &[Token<T>] {
        &self.after
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn range(&self) -> &Range {
        &self.range
    }
}

pub struct Table<T> {
    pub(super) table: Vec<HashMap<Item<T>, InsertedBy>>,
    grammar: Grammar<T>,
    pub(super) initial: String,
}

impl<T> Table<T>
where
    T: Clone + std::cmp::Eq + std::hash::Hash + Display,
{
    pub fn new(
        grammar: Grammar<T>,
        initial: impl AsRef<str>,
        size_hint: usize,
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
        let table = Vec::with_capacity(size_hint + 1);
        let mut out = Table {
            table,
            grammar,
            initial: initial.as_ref().to_owned(),
        };
        let initials = out.pred_phase(0, initials);
        out.table.push(initials);
        out
    }

    #[allow(dead_code)]
    pub fn print_table(&self) {
        for (j, el) in self.table.iter().enumerate() {
            for it in el.keys() {
                println!("inserting at {j}: {it}");
            }
            println!();
        }
    }

    fn scan_phase(&mut self, j: usize, token: T) -> HashMap<Item<T>, InsertedBy> {
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
        cur_state: &HashMap<Item<T>, InsertedBy>,
    ) -> Vec<(Item<T>, InsertedBy)> {
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
        mut cur_state: HashMap<Item<T>, InsertedBy>,
    ) -> HashMap<Item<T>, InsertedBy> {
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
        mut cur_state: HashMap<Item<T>, InsertedBy>,
    ) -> HashMap<Item<T>, InsertedBy> {
        loop {
            let added = self.pred_phase_loop(j, &cur_state);
            let mut num_added = 0;
            for (add, inserted_by) in added {
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
        cur_state: &HashMap<Item<T>, InsertedBy>,
    ) -> Vec<(Item<T>, InsertedBy)> {
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

    pub(super) fn next(&mut self, token: T) {
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

        // TODO: fix empty productions
        // an empty production can be applied immediately after the `pred` phase, at which point,
        // it could introduce more reductions for the comp_phase


        self.table.push(cur_state);
    }

    
}
