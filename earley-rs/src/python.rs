use std::collections::HashSet;

use crate::Grammar as Gram;
use itertools::Itertools;
use pyo3::{create_exception, prelude::*};

#[pyclass]
#[derive(Clone)]
pub struct Token {
    inner: crate::Token<String>,
}

#[pyfunction]
#[pyo3(name = "Term")]
fn term(s: &str) -> Token {
    Token::t(s.to_string())
}

#[pyfunction]
#[pyo3(name = "NonTerm")]
fn non_term(s: &str) -> Token {
    Token::nt(s.to_string())
}

#[pymethods]
impl Token {
    /// constructs a new terminal token
    #[staticmethod]
    fn t(s: String) -> Self {
        Self {
            inner: crate::Token::Term(s),
        }
    }

    /// constructs a new non-terminal token
    #[staticmethod]
    fn nt(s: String) -> Self {
        Self {
            inner: crate::Token::NonTerm(s),
        }
    }

    fn __str__(slf: PyRef<'_, Self>) -> String {
        slf.inner.to_string()
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self.inner)
    }
}

#[pyclass]
#[derive(Clone)]
pub struct Grammar {
    inner: Gram<String>,
}

#[pymethods]
impl Grammar {
    #[new]
    fn new() -> Self {
        Grammar { inner: Gram::new() }
    }

    fn add_prod(&mut self, nonterm: &str, expansion: Vec<Token>) {
        self.inner
            .add_prod(nonterm, expansion.into_iter().map(|x| x.inner));
    }

    fn __str__(&self) -> String {
        let mut out = String::new();
        self.str(&mut out, 0);
        out
    }

    fn __repr__(&self) -> String {
        let mut out = String::from("Grammar (\n");
        self.str(&mut out, 4);
        out.push(')');
        out
    }
}

impl Grammar {
    fn str(&self, out: &mut String, indent: usize) {
        use std::fmt::Write;
        for (rule, exp) in &self.inner.productions {
            for exp in exp {
                for _ in 0..indent {
                    out.push(' ');
                }
                writeln!(out, "{rule} -> {}", exp.iter().format(" ")).unwrap();
            }
        }
    }
}

#[pyclass]
pub struct PrefixParser {
    inner: crate::PrefixParser<String>,
    progress: String,
}

create_exception!(earley, ParseError, pyo3::exceptions::PyException);

impl From<crate::Error> for PyErr {
    fn from(value: crate::Error) -> Self {
        ParseError::new_err(value.to_string())
    }
}

#[pymethods]
impl PrefixParser {
    #[new]
    pub fn new(grammar: Grammar, initial: &str) -> Self {
        let inner = crate::PrefixParser::new(grammar.inner, initial);
        Self {
            inner,
            progress: String::new(),
        }
    }

    pub fn try_next(&mut self, token: &str) -> PyResult<()> {
        self.inner.try_next(token.to_owned())?;
        self.progress += token;
        Ok(())
    }

    pub fn finish(&self) -> PyResult<()> {
        self.inner.finish()?;
        Ok(())
    }

    pub fn __repr__(&self) -> String {
        format!(
            "PrefixParser(progress={:?}, <parser@0x{:x}>)",
            self.progress,
            (self as *const _) as usize
        )
    }

    pub fn legal_tokens(&self) -> HashSet<String> {
        self.inner.legal_tokens()
    }

    #[getter]
    pub fn progress(&self) -> String {
        self.progress.clone()
    }
}

#[pymodule]
pub fn earley(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PrefixParser>()?;
    m.add_class::<Grammar>()?;
    m.add_class::<Token>()?;
    m.add("ParseError", py.get_type_bound::<ParseError>())?;
    m.add_function(wrap_pyfunction!(term, m)?)?;
    m.add_function(wrap_pyfunction!(non_term, m)?)?;
    Ok(())
}
