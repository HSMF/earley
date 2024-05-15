from typing import List, Set, overload


class Token:
    @staticmethod
    def t(s: str) -> Token: ...

    @staticmethod
    def nt(s: str) -> Token: ...


def Term(s: str) -> Token:
    ...

def NonTerm(s: str) -> Token:
    ...

class Grammar:
    @overload
    def __new__(cls) -> Grammar: ...

    @overload
    def __new__(cls) -> Grammar: ...

    def add_prod(self, nonterm: str, expansion: List[Token]): ...


class PrefixParser:
    progress: str

    @overload
    def __new__(cls, grammar: Grammar, initial: str) -> PrefixParser: ...

    @overload
    def __new__(cls) -> Grammar: ...

    def try_next(self, token: str): ...

    def finish(self): ...

    def legal_tokens(self) -> Set[str]: ...

class ParseError(Exception):
    ...
