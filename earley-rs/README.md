# Earley-rs

![rewrite it in rust](https://imgur.com/cEzxFOC.jpg)

```sh
cargo run
```

Currently you will have to edit [`src/main.rs`](./src/main.rs) to change the grammar

## Python bindings


1. create a virtual environment
  ```sh
  python -m venv .env
  source .env/bin/activate
  ```

2. install build requirements
  ```sh
  pip install maturin
  ```

3. build and install the early module into the current venv
  ```sh
  maturin develop
  ```

4. use `earley` in code!

  For a more fleshed out example, see [lua.py](./lua.py)

  ```python
  from earley import Grammar, Token, PrefixParser, ParseError, Term, NonTerm

  # this is the grammar for balanced parentheses
  g = Grammar()
  g.add_prod("S", [Token.nt("S"), Token.nt("S")])
  # Term is an alias for Token.t, NonTerm is an alias for Token.nt
  g.add_prod("S", [Term("("), NonTerm("S"), Term(")")])
  g.add_prod("S", [Token.t("("), Token.t(")")])

  p = PrefixParser(g, "S")

  p.try_next("(")
  p.try_next(")")
  print(f"so far we have parsed {p.progress}!")

  try:
    p.try_next(")")
  except ParseError as pe:
    print("could not add ) because none was opened")

  print(p.legal_tokens())
  ```
