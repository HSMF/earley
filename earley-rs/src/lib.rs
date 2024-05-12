use std::{collections::HashMap, fmt::Display};

use table::Item;

use self::latex::Proof;

pub mod latex;
mod table;
pub use table::Table;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Token<T> {
    Term(T),
    NonTerm(String),
}

impl<T> Token<T> {
    fn nonterm(&self) -> &str {
        match self {
            Token::NonTerm(s) => s,
            _ => panic!(),
        }
    }

    fn term(&self) -> &T {
        match self {
            Token::Term(s) => s,
            _ => panic!(),
        }
    }
}

impl<T> Display for Token<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Term(t) => write!(f, "`{t}`"),
            Token::NonTerm(t) => write!(f, "{t}"),
        }
    }
}

pub fn t(s: impl ToString) -> Token<String> {
    Token::Term(s.to_string())
}
pub fn nt(s: impl ToString) -> Token<String> {
    Token::NonTerm(s.to_string())
}

pub type Range = std::ops::Range<usize>;

#[derive(Debug, Clone)]
pub struct Grammar<T> {
    productions: HashMap<String, Vec<Vec<Token<T>>>>,
}

impl<T> Grammar<T> {
    pub fn new() -> Self {
        Self {
            productions: HashMap::new(),
        }
    }

    pub fn add_prod(
        &mut self,
        nonterm: impl ToString,
        expansion: impl IntoIterator<Item = Token<T>>,
    ) {
        let nonterm = nonterm.to_string();
        let expansion = expansion.into_iter().collect();
        let entry = self.productions.entry(nonterm).or_default();
        entry.push(expansion);
    }

    pub fn latex(&self) -> latex::Grammar<T> {
        latex::Grammar(self)
    }
}

impl<T> Default for Grammar<T> {
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

pub struct Parser<T, I> {
    input: I,
    table: Table<T>,
}

impl<T, I> Parser<T, I>
where
    I: Iterator<Item = T>,
    T: Clone + Eq + std::hash::Hash + Display,
{
    pub fn new(
        input: impl IntoIterator<IntoIter = I>,
        grammar: Grammar<T>,
        initial: impl AsRef<str>,
    ) -> Self {
        let input = input.into_iter();
        let table = Table::new(grammar, initial, input.size_hint().0);
        Self { input, table }
    }
    pub fn parse(mut self) -> Result<ParseInfo<T>, Error> {
        for token in self.input.by_ref() {
            // println!("{token}");
            self.table.next(token)
        }
        // self.table.print_table();

        let initial = &self.table.initial;

        let _ = self
            .table
            .table
            .last()
            .unwrap()
            .iter()
            .find(|(item, _)| {
                item.name() == initial
                    && item.after().is_empty()
                    && item.range() == &(0..self.table.table.len() - 1)
            })
            .ok_or(Error)?;
        Ok(ParseInfo {
            table: self.table.table,
            initial: self.table.initial,
        })
    }
}

// pub struct PrefixParser<T> {}

pub struct ParseInfo<T> {
    table: Vec<HashMap<Item<T>, InsertedBy>>,
    initial: String,
}

#[derive(Debug, thiserror::Error)]
#[error("oops")]
pub struct Error;

impl<T> ParseInfo<T>
where
    T: PartialEq + Display,
{
    fn reconstruct_tree<'a>(
        &'a self,
        j: usize,
        root: &'a Item<T>,
        inserted_by: &InsertedBy,
    ) -> latex::Proof<'a, T> {
        match root.before().last() {
            None => {
                assert!(matches!(inserted_by, InsertedBy::Pred));
                Proof::Pred(root)
            }
            Some(t @ Token::Term(_)) => {
                assert!(matches!(inserted_by, InsertedBy::Scan(..)));
                let mu_range = inserted_by.clone().scan_range();
                let mu = &root.before()[..root.before().len().saturating_sub(1)];
                let child = self.table[j - 1]
                    .iter()
                    .find(|(x, _)| {
                        x.after().last() == Some(t) && mu == x.before() && x.range() == &mu_range
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
                    .find(|(x, _)| x.name() == t && x.after().is_empty() && x.range() == &range_b)
                    .expect("child B must end at the same point");
                let index_mu = child_b.0.range().start;
                let name_of_b = Token::NonTerm(child_b.0.name().to_owned());
                let child_mu = &self.table[index_mu]
                    .iter()
                    .find(|(x, _)| {
                        x.name() == root.name()
                            && x.after().last() == Some(&name_of_b)
                            && x.range() == &range_mu
                    })
                    .expect("there's a mu item before somewhere");
                let proof_b = self.reconstruct_tree(j, child_b.0, child_b.1);
                let proof_mu = self.reconstruct_tree(index_mu, child_mu.0, child_mu.1);
                Proof::Comp(root, Box::new(proof_mu), Box::new(proof_b))
            }
        }
    }

    pub fn reconstruct(&self) -> latex::FullProof<T> {
        let initial: &str = self.initial.as_ref();
        let root = self
            .table
            .last()
            .unwrap()
            .iter()
            .find(|(item, _)| item.name() == initial && item.after().is_empty())
            .expect("parse had failed. if you see this you may complain about this horrid api");

        let proof = self.reconstruct_tree(self.table.len() - 1, root.0, root.1);
        latex::FullProof(proof)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const S: &str = "INIT";
    // grammars for tests operate on strings, mainly because it's sort of easy

    /// <P> ::= <S>      # the start rule
    /// <S> ::= <S> "+" <M> | <M>
    /// <M> ::= <M> "*" <T> | <T>
    /// <T> ::= "1" | "2" | "3" | "4"
    fn factored_arith() -> Grammar<String> {
        // https://en.wikipedia.org/wiki/Earley_parser#Example
        let mut grammar = Grammar::new();
        grammar.add_prod("P", [nt("S")]);

        grammar.add_prod("S", [nt("S"), t('+'), nt("M")]);
        grammar.add_prod("S", [nt("M")]);

        grammar.add_prod("M", [nt("M"), t('*'), nt("T")]);
        grammar.add_prod("M", [nt("T")]);

        grammar.add_prod("T", [t('1')]);
        grammar.add_prod("T", [t('2')]);
        grammar.add_prod("T", [t('3')]);
        grammar.add_prod("T", [t('4')]);
        grammar
    }

    /// the grammar
    /// S => aSa | bSb | \eps
    /// is context-free. It is not proper since it includes an ε-production.
    ///
    /// see [https://en.wikipedia.org/wiki/Context-free_grammar]
    fn improper_rev() -> Grammar<String> {
        let mut grammar = Grammar::new();

        grammar.add_prod("INIT", [t('a'), nt("INIT"), t('a')]);
        grammar.add_prod("INIT", [t('b'), nt("INIT"), t('b')]);
        grammar.add_prod("INIT", []);

        grammar
    }

    /// S ::= aSa | bSb | \eps | a | b
    ///
    /// see [https://en.wikipedia.org/wiki/Context-free_grammar]
    fn palindrome() -> Grammar<String> {
        let mut grammar = Grammar::new();

        grammar.add_prod("INIT", [t('a'), nt("INIT"), t('a')]);
        grammar.add_prod("INIT", [t('b'), nt("INIT"), t('b')]);
        grammar.add_prod("INIT", [t('b')]);
        grammar.add_prod("INIT", [t('a')]);
        grammar.add_prod("INIT", []);

        grammar
    }

    fn well_formed_parentheses() -> Grammar<String> {
        let mut grammar = Grammar::new();
        grammar.add_prod(S, [nt(S), nt(S)]);
        grammar.add_prod(S, [t("("), nt(S), t(")")]);
        grammar.add_prod(S, [t("("), t(")")]);
        grammar
    }

    fn input(x: &str) -> impl Iterator<Item = String> + '_ {
        x.chars()
            .filter(|x| !x.is_whitespace())
            .map(|x| x.to_string())
    }

    macro_rules! test_grammar {
        ($name:ident, $grammar:expr, $input:expr) => {
            test_grammar!($name, $grammar, $input, "INIT");
        };
        ($name:ident, $grammar:expr, $input:expr, $initial:expr) => {
            #[test]
            fn $name() {
                let grammar = $grammar();
                let input = input($input);
                let parser = Parser::new(input, grammar, $initial);
                let result = parser.parse();
                assert!(result.is_ok());
            }
        };
        (FAIL $name:ident, $grammar:expr, $input:expr) => {
            test_grammar!(FAIL $name, $grammar, $input, "INIT");
        };
        (FAIL $name:ident, $grammar:expr, $input:expr, $initial:expr) => {
            #[test]
            fn $name() {
                let grammar = $grammar();
                let input = input($input);
                let parser = Parser::new(input, grammar, $initial);
                let result = parser.parse();
                assert!(result.is_err());
            }
        };
    }

    test_grammar!(factored_arith1, factored_arith, "2 + 3 * 4", "P");
    test_grammar!(FAIL factored_arith2, factored_arith, "2 + * 4", "P");
    test_grammar!(
        factored_arith3,
        factored_arith,
        "1 + 2 + 3 + 4 * 3 * 2 * 1 + 1",
        "P"
    );

    test_grammar!(improper_rev_empty, improper_rev, "");
    test_grammar!(improper_rev_easy, improper_rev, "aabbaa");
    test_grammar!(FAIL improper_rev_easy_fail, improper_rev, "aabaa");

    test_grammar!(palindrome_empty, palindrome, "");
    test_grammar!(palindrome_easy, palindrome, "aabbaa");
    test_grammar!(palindrome_not_rev_concat, palindrome, "aabaa");
    test_grammar!(FAIL palindrome_easy_fail, palindrome, "abaa");
    test_grammar!(FAIL palindrome_easy_fail2, palindrome, "ab");

    test_grammar!(well_formed_parentheses1, well_formed_parentheses, "()");
    test_grammar!(
        well_formed_parentheses2,
        well_formed_parentheses,
        "(((((())))))"
    );
    // test_grammar!(FAIL well_formed_parentheses_fail1, well_formed_parentheses, "((((())))))");
    test_grammar!(FAIL well_formed_parentheses_fail2, well_formed_parentheses, "((((())))");
}
