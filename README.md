# Earley Parser

## Overview

- [the caml directory](./caml/) contains a some naive OCaml code that implements an earley parser, badly
- [the earley-rs directory](./earley-rs/) contains a Rust rewrite, which is naturally better

Python bindings are available in the rust implementation


## Running - OCaml

Requirements:
- dune
- ocamlc `5.1.0` or later (earlier may work too)

```sh
cd caml
dune exec earley
```

## Running - Rust

Requirements:
- rust `1.77.2`, or later
- cargo `1.77.2`, or later


```sh
cd earley-rs

cargo run
```
