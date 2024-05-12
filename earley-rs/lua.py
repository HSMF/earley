"""
this is a fairly poor but incremental lua parser

* currently, since empty productions are not yet supported, trailing fieldsep's are not allowed
* currently, the way we handle semicolons is by just trying to insert one if we get a parse error
* strings are not supported
* numbers and idents are kinda wonky
* spaces are mostly ignored (so hello world and helloworld are not distinguishable)
"""

from typing import List, Set, Tuple
from earley import (
    Grammar,  # type: ignore
    PrefixParser,  # type: ignore
    ParseError,  # type: ignore
    Term,  # type: ignore
    NonTerm,  # type: ignore
)

from string import ascii_lowercase


g = Grammar()

_unique = 0


def unique() -> str:
    global _unique
    _unique += 1
    return f"rule{_unique}"


def add(spec):
    for r, exp in spec:
        g.add_prod(r, exp)


def alt(prods, nonterm):
    nonterm = nonterm if nonterm is not None else unique()
    return [(nonterm, prod) for prod in prods]


def rep(basecase, iteration, nonterm=None):
    nonterm = nonterm if nonterm is not None else unique()
    return [
        (nonterm, basecase),
        (nonterm, [nt(nonterm)] + iteration),
    ]


def seq(spec1, spec2, nonterm=None):
    nonterm = nonterm if nonterm is not None else unique()

    names = set([i for i, _ in spec1] + [i for i, _ in spec2])

    return spec1 + spec2 + [(nonterm, [nt(name)]) for name in names]


# g.add_prod("S", [])

t = Term
nt = NonTerm

for dig in range(0, 10):
    g.add_prod("digit", [t(str(dig))])

for ch in ascii_lowercase:
    g.add_prod("letter", [t(ch)])

g.add_prod("number", [nt("digit")])
g.add_prod("number", [nt("number"), nt("digit")])

g.add_prod("Name", [nt("letter")])
g.add_prod("Name", [nt("Name"), nt("letter")])

# https://parrot.github.io/parrot-docs0/0.4.7/html/languages/lua/doc/lua51.bnf.html

add(alt(nonterm="unop", prods=[[t(unop)] for unop in ("#", "-", "not")]))
add(
    alt(
        nonterm="binop",
        prods=[
            [t(binop)]
            for binop in (
                "+",
                "-",
                "*",
                "/",
                "^",
                "%",
                "..",
                "<",
                "<=",
                ">",
                ">=",
                "==",
                "~=",
            )
        ],
    )
)

add(alt(nonterm="fieldsep", prods=[[t(i)] for i in (",", ";")]))

add(
    rep([nt("field")], [nt("fieldsep"), nt("field")], nonterm="fieldlist")
)  # TODO: add optional trailing fieldsep once eps is supported


add(
    alt(
        nonterm="field",
        prods=[
            [t("["), nt("exp"), t("]"), t("="), nt("exp")],
            [t("Name"), t("="), nt("exp")],
            [nt("exp")],
        ],
    )
)

add(
    alt(
        nonterm="exp",
        prods=[[t(root)] for root in ("nil", "false", "true", "...")]
        + [
            [nt("number")],
            [nt("exp"), nt("binop"), nt("exp")],
            [nt("unop"), nt("exp")],
            [nt("function")],
            [nt("prefixexp")],
            [nt("tableconstructor")],
        ],
        # string
    )
)

g.add_prod("tableconstructor", [t("{"), t("}")])
g.add_prod("tableconstructor", [t("{"), nt("fieldlist"), t("}")])


g.add_prod("parlist1", [nt("namelist")])
g.add_prod("parlist1", [t("...")])
g.add_prod("parlist1", [nt("..."), t(","), t("...")])

g.add_prod("funcbody", [t("("), t(")"), nt("block"), t("end")])
g.add_prod("funcbody", [t("("), nt("parlist1"), t(")"), nt("block"), t("end")])

g.add_prod("function", [t("function"), nt("funcbody")])

add(
    alt(
        nonterm="args",
        prods=[
            [t("("), t(")")],
            [t("("), nt("explist1"), t(")")],
            [nt("tableconstructor")],
            # string
        ],
    )
)

add(
    alt(
        nonterm="functioncall",
        prods=[
            [nt("prefixexp"), nt("args")],
            [nt("prefixexp"), t(":"), nt("Name"), nt("args")],
        ],
    )
)

add(
    alt(
        nonterm="prefixexp",
        prods=[
            [nt("var")],
            [nt("functioncall")],
            [t("("), nt("exp"), t(")")],
        ],
    )
)

add(rep(basecase=[nt("exp")], iteration=[t(","), nt("exp")], nonterm="explist1"))

add(rep(basecase=[nt("Name")], iteration=[t(","), nt("Name")], nonterm="namelist"))

add(rep(basecase=[nt("var")], iteration=[t(","), nt("var")], nonterm="varlist"))

add(
    alt(
        nonterm="var",
        prods=[
            [nt("Name")],
            [nt("prefixexp"), t("["), nt("exp"), t("]")],
            [nt("prefixexp"), t("."), nt("Name")],
        ],
    )
)

fnpath = unique()
g.add_prod("funcname", [nt(fnpath)])
g.add_prod("funcname", [nt(fnpath), t(":"), nt("Name")])

add(rep(nonterm=fnpath, basecase=[nt("Name")], iteration=[t("."), nt("Name")]))

g.add_prod("laststat", [t("break")])
g.add_prod("laststat", [t("return")])
g.add_prod("laststat", [t("return"), nt("explist1")])

elseifchain = unique()
add(
    rep(
        nonterm=elseifchain,
        basecase=[t("if"), nt("exp"), t("then"), nt("block")],
        iteration=[t("elseif"), nt("exp"), t("then"), nt("block")],
    )
)

add(
    alt(
        nonterm="stat",
        prods=[
            [nt("varlist"), t("="), nt("explist1")],
            [nt("functioncall")],
            [t("do"), nt("block"), t("end")],
            [t("while"), nt("exp"), t("do"), nt("block"), t("end")],
            [t("repeat"), nt("block"), t("until"), nt("exp")],
            [nt(elseifchain), t("end")],
            [nt(elseifchain), t("else"), nt("block"), t("end")],
            [
                t("for"),
                nt("Name"),
                t("="),
                nt("exp"),
                t(","),
                nt("exp"),
                t("do"),
                nt("block"),
                t("end"),
            ],
            [
                t("for"),
                nt("Name"),
                t("="),
                nt("exp"),
                t(","),
                nt("exp"),
                t(","),
                nt("exp"),
                t("do"),
                nt("block"),
                t("end"),
            ],
            [
                t("for"),
                nt("namelist"),
                t("in"),
                nt("explist1"),
                t("do"),
                nt("block"),
                t("end"),
            ],
            [t("function"), nt("funcname"), nt("funcbody")],
            [t("local"), t("function"), nt("Name"), nt("funcbody")],
            [t("local"), nt("namelist")],
            [t("local"), nt("namelist"), t("="), nt("explist1")],
        ],
    )
)

g.add_prod("block", [nt("chunk")])


add(rep(nonterm="stats", basecase=[nt("stat"), t(";")], iteration=[nt("stat"), t(";")]))

g.add_prod("chunk", [nt("stats")])
g.add_prod("chunk", [nt("stats"), nt("laststat"), t(";")])

# print(g)

p = PrefixParser(g, initial="chunk")


def push(tok: str):
    try:
        p.try_next(tok)
        print(p.legal_tokens())
    except ParseError as pe:
        print(f"could not add {tok}: {pe}")
        print(f"trying to insert semicolon")
        try:
            p.try_next(";")
            p.try_next(tok)
        except ParseError as pe:
            print(f"still does not work {pe}")

keywords = {
    "do",
    "end",
    "while",
    "repeat",
    "until",
    "if",
    "then",
    "elseif",
    "else",
    "for",
    "in",
    "local",
    "function",
    "return",
    "break",
    "nil",
    "false",
    "true",
    "and",
    "or",
    "not",
}
multichar_op = {"...", "<=", ">=", "==", "~=", ".."}


def to_tokens(s: str) -> List[Tuple[str, int, int]]:
    def multichar(s: str, refset: Set[str]):
        for kw in refset:
            if s.startswith(kw):
                return kw

        return None

    out = []
    s = s.rstrip()

    i = 0
    while len(s) > 0:
        s2 = s.lstrip()
        i += len(s) - len(s2)
        s = s2

        kw = multichar(s, keywords)
        if kw is not None:
            out.append((kw, i, i + len(kw)))
            i += len(kw)
            s = s[len(kw) :]
            continue

        op = multichar(s, multichar_op)
        if op is not None:
            out.append((op, i, i + len(op)))
            i += len(op)
            s = s[len(op) :]
            continue

        out.append((s[0], i, i + 1))
        i += 1
        s = s[1:]

    return out


print(p)
string = "local x = 10\nlocal y = local"
for token, start, end in to_tokens(string):
    push(token)
    print(string[:end])
