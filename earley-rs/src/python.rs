use crate::Grammar as Gram;
use itertools::Itertools;
use pyo3::prelude::*;

#[pyclass]
pub struct Token {
    inner: crate::Token<String>,
}

#[pymethods]
impl Token {
    #[staticmethod]
    fn t(s: String) -> Self {
        Self {
            inner: crate::Token::Term(s),
        }
    }

    #[staticmethod]
    fn nt(s: String) -> Self {
        Self {
            inner: crate::Token::NonTerm(s),
        }
    }

    fn __str__(slf: PyRef<'_, Self>) -> String {
        slf.inner.to_string()
    }

    fn __repr__(slf: PyRef<'_, Self>) -> String {
        slf.inner.to_string()
    }
}

#[pyclass]
pub struct PrefixParser {
    inner: crate::PrefixParser<String>,
}

#[pyclass]
pub struct Grammar {
    inner: Gram<String>,
}

#[pymethods]
impl Grammar {
    #[new]
    fn new() -> Self {
        Grammar { inner: Gram::new() }
    }

    fn add_prod(&mut self, nonterm: &str, expansion: Vec<&Token>) {
        println!("hey");
        // self.inner
        //     .add_prod(nonterm, expansion.into_iter().map(|x| x.inner));
    }

    fn __str__(slf: PyRef<'_, Self>) -> String {
        let mut builder = String::new();
        for (rule, exp) in &slf.inner.productions {
            for exp in exp {
                builder = format!("{builder}{rule} -> {}\n", exp.iter().format(" "));
            }
        }
        builder
    }

    fn __repr__(slf: PyRef<'_, Self>) -> String {
        Grammar::__str__(slf)
    }
}

#[pymodule]
pub fn earley(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PrefixParser>()?;
    m.add_class::<Grammar>()?;
    m.add_class::<Token>()?;
    Ok(())
}
