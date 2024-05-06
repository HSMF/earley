open! Util
open! Printf

let p s = print_endline s
let indent = indent ~s:"  "
let pi depth s = p (indent depth ^ s)

type span_index =
  | Known of int
  | Infer of (int * int)

type span = span_index * span_index

let assert_known = function
  | Infer _ -> failwith "i literally dont know"
  | Known i -> i


let span_lower_bound = function
  | Known i | Infer (i, _) -> i


let span_upper_bound = function
  | Known i | Infer (_, i) -> i


let bound_below by = function
  | Known i -> Known i
  | Infer (i, j) -> Infer (max by i, j)


let bound_above by = function
  | Known i -> Known i
  | Infer (i, j) -> Infer (i, min by j)


let specify i j =
  match i, j with
  | Known i, Known j ->
    p @@ sp "Known %d = Known %d" i j;
    assert (i = j);
    Known i
  | Infer _, Known j -> Known j
  | Known i, Infer _ -> Known i
  | Infer (i, j), Infer (i', j') -> Infer (max i i', min j j')


type token =
  | Term of string
  | NonTerm of string

type production = string * token list

type item =
  | Production of production
  | PrsLiteral of span * string
  | PrsProduction of span * string * token list * token list

type grammar = production list

let string_of_span_index = function
  | Known i -> sp "%d" i
  | Infer (a, b) -> sp "?(%d, %d)" a b


let string_of_span ((a, b) : span) =
  sp "%s, %s" (string_of_span_index a) (string_of_span_index b)


let string_of_token = function
  | Term s -> sp "`%s`" s
  | NonTerm s -> s


let string_of_tokens = sl string_of_token " "

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


type proof_rule =
  | AxiomProduction of string * token list
  | AxiomToken of span * string
  | Pred of item * proof_rule
  | Scan of item * proof_rule * proof_rule
  | Comp of item * proof_rule * proof_rule

type proof = span * proof_rule

let latex_of_token : token -> string = function
  | Term s -> sp "'\\texttt{%s}`" s
  | NonTerm s -> sp {|\textit{%s}|} s


let latex_of_item : item -> string = function
  | Production _ | PrsLiteral (_, _) ->
    failwith "todo p case Production _ | PrsLiteral (_, _)"
  | PrsProduction (span, prod, before, after) ->
    sp
      {|[%s, %s \ensuremath{\to} %s \ensuremath{\bullet} %s]|}
      (* {|[%s, %s to %s circ %s]|} *)
      (string_of_span span)
      prod
      (sl latex_of_token " " (List.rev before))
      (sl latex_of_token " " after)


let rec string_of_proof_rule : proof_rule -> string = function
  | AxiomProduction (r, expansion) ->
    sp
      {|\axiominf{%s \ensuremath{\to} %s}{\texttt{gram}}|}
      r
      (sl latex_of_token " " expansion)
  | AxiomToken ((i, _), tok) ->
    sp
      {|\axiominf{\ensuremath{x_{%d}} = %s}{\texttt{src}}|}
      (assert_known i)
      (latex_of_token (Term tok))
  | Pred (item, rule) ->
    sp
      {|\uninf{%s}{\texttt{Pred}}{
      %s
      }|}
      (latex_of_item item)
      (string_of_proof_rule rule)
  | Scan (item, rule_mu, rule_a) ->
    sp
      {|\bininf{%s}{\texttt{Scan}}{
      %s
      }{
      %s
      }|}
      (latex_of_item item)
      (string_of_proof_rule rule_mu)
      (string_of_proof_rule rule_a)
  | Comp (item, rule_mu, rule_b) ->
    sp
      {|\bininf{%s}{\texttt{Comp}}{
      %s
      }{
      %s
      }
      |}
      (latex_of_item item)
      (string_of_proof_rule rule_mu)
      (string_of_proof_rule rule_b)


let string_of_proof (_, proof) =
  let prelude =
    {|\documentclass{article}\usepackage{bussproofs}\usepackage{hyde}
\usepackage{incgraph}
\begin{document}

    \newenvironment{bprooftree}
  {\leavevmode\hbox\bgroup}
  {\DisplayProof\egroup}
    \begin{inctext}[left border=20pt, right border=20pt,top border=30pt, bottom border=30pt]
    \begin{bprooftree}
    |}
  in
  let postlude = {|
    \end{bprooftree}

    \end{inctext}
    \end{document} |} in
  prelude ^ string_of_proof_rule proof ^ postlude


let possible_expansions (grammar : grammar) (nonterm : string) =
  List.filter_map (fun (k, v) -> Option.then_some (k = nonterm) v) grammar


type input = string list

(** gets the input at the span *)
let ( @. ) input ((i, j) : span) =
  input |> List.drop (assert_known i) |> List.take (assert_known j - assert_known i)


exception ParseError

let rec any depth f = function
  | [] -> None
  | x :: xs -> begin
    try Option.some @@ f x with
    | ParseError ->
      pi depth @@ sp "- failure";
      any depth f xs
  end


let mk_production (span, name, before, after) =
  if List.is_empty before
  then Production (name, after)
  else PrsProduction (span, name, before, after)


let rec comp depth grammar (i, k) input name mu b ni : proof =
  pi depth @@ sp "↳ comp %s" (string_of_span (i, k));
  let depth = depth + 1 in
  let j =
    if List.is_empty mu then i else Infer (span_lower_bound i, span_upper_bound k)
  in
  let can_prove_b =
    any
      depth
      (fun prod ->
        prove_item depth grammar (mk_production ((j, k), b, List.rev prod, [])) input)
      (possible_expansions grammar b)
  in
  let (j, k'), proof_b =
    begin
      match can_prove_b with
      | None -> raise ParseError
      | Some span -> span
    end
  in
  let j = assert_known j in
  let span, proof_mu =
    prove_item
      depth
      grammar
      (mk_production ((i, Known j), name, mu, NonTerm b :: ni))
      input
  in
  let i', _ = if List.is_empty mu then Known j, Known j else span in
  let i = specify i i' in
  let span = i, specify k k' in
  let orig_item = PrsProduction (span, name, NonTerm b :: mu, ni) in
  span, Comp (orig_item, proof_mu, proof_b)


and scan depth grammar (i, k) input name mu a ni : proof =
  let depth = depth + 1 in
  pi depth @@ sp "↳ scan %s" (string_of_span (i, k));
  let i = bound_below (assert_known k - 1) i in
  let j = Known (assert_known k - 1) in
  let _, proof_a = prove_item depth grammar (PrsLiteral ((j, k), a)) input in
  let item =
    if List.is_empty mu
    then Production (name, Term a :: ni)
    else PrsProduction ((i, j), name, mu, Term a :: ni)
  in
  let (i', _), proof_mu = prove_item depth grammar item input in
  let span = if List.is_empty mu then j, k else specify i i', k in
  pi depth @@ sp "↳ end scan %s" (string_of_span span);
  span, Scan (PrsProduction (span, name, Term a :: mu, ni), proof_mu, proof_a)


and pred depth grammar (i, j) input name expansion : proof =
  let depth = depth + 1 in
  pi depth @@ "↳ pred ";
  if i <> j then raise @@ ParseError;
  let span, proof =
    prove_item depth grammar (Production (name, List.rev expansion)) input
  in
  span, Pred (PrsProduction (span, name, [], expansion), proof)


and prove_item depth grammar (it : item) (input : input) : proof =
  let n = List.length input in
  match it with
  | Production (name, expansion) ->
    let expansions = possible_expansions grammar name in
    if List.for_all (fun x -> expansion <> x) expansions
    then raise @@ ParseError
    else (Infer (0, n + 1), Infer (0, n + 1)), AxiomProduction (name, expansion)
  | PrsLiteral (span, item) ->
    pi depth @@ sp "[%s] = [%s] %s" item (sl id "," (input @. span)) (string_of_span span);
    if input @. span <> [ item ] then raise ParseError else span, AxiomToken (span, item)
  | PrsProduction (span, name, before, after) ->
    pi depth @@ sp "↳ we prove production @ %s " (string_of_item it);
    assert (not (List.is_empty before));
    let res =
      begin
        match before with
        | [] -> pred depth grammar span input name after
        | Term a :: mu -> scan depth grammar span input name mu a after
        | NonTerm b :: mu -> comp depth grammar span input name mu b after
      end
    in
    res
