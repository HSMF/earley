open! Util

type span = int * int

type token =
  | Term of string
  | NonTerm of string

type production = string * token list

type item =
  | Production of production
  | PrsLiteral of span * string
  | PrsProduction of span * string * token list * token list

type grammar = production list

let string_of_span ((a, b) : span) = sp "%d, %d" a b

let string_of_token = function
  | Term s -> sp "`%s`" s
  | NonTerm s -> s


let string_of_item = function
  | Production (s, toks) -> sp "%s -> %s" s (sl string_of_token " " toks)
  | PrsLiteral (span, lit) -> sp "[%s, `%s`]" (string_of_span span) lit
  | PrsProduction (span, prod, before, after) ->
    sp
      "[%s, %s -> %s ● %s]"
      (string_of_span span)
      prod
      (sl string_of_token " " (List.rev before))
      (sl string_of_token " " after)


let my_grammar : grammar =
  [ "P", [ NonTerm "S" ]
  ; "S", [ NonTerm "S"; Term "+"; NonTerm "M" ]
  ; "S", [ NonTerm "M" ]
  ; "M", [ NonTerm "M"; Term "*"; NonTerm "T" ]
  ; "M", [ NonTerm "T" ]
  ; "T", [ Term "1" ]
  ; "T", [ Term "2" ]
  ; "T", [ Term "3" ]
  ; "T", [ Term "4" ]
  ]


type input = string list

let input = "2 + 3 * 4" |> String.split_on_char ' '

(** gets the input at the span *)
let ( @. ) input ((i, j) : span) = input |> List.drop i |> List.take (j - i)

exception ParseError

let comp = failwith "todo"
let scan = failwith "todo"
let pred = failwith "todo"

let prove_item (it : item) (input : input) =
  match it with
  | Production (name, toks) -> failwith "todo"
  | PrsLiteral (span, item) -> if input @. span <> [ item ] then raise ParseError
  | PrsProduction (span, name, before, after) ->
    assert (not (List.is_empty before));
    let to_prove = List.hd before in
    let previous = List.tl before in
    begin
      match to_prove with
      | Term _ | NonTerm _ -> failwith "todo"
    end


