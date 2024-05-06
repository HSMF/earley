open Lib
open Printf

let input : input = "2 + 3 * 1 + 1 * 2 * 1" |> String.split_on_char ' '
let entry = "P"
let entry_expansion = List.assoc entry my_grammar |> List.rev
let goal = PrsProduction ((Known 0, Known (List.length input)), entry, entry_expansion, [])
let () = print_newline ()
let () = printf "input: %s\n" (String.concat " " input)
let () = printf "trying to prove %s\n" (string_of_item goal)
let span, proof = prove_item 0 my_grammar goal input
let () = printf "proved %s to have span %s\n" (string_of_item goal) (string_of_span span)
let latex = string_of_proof (span, proof)
let f = open_out "output.tex"

let () =
  output_string f latex;
  close_out f


let () = ignore @@ Sys.command "pdflatex -output-directory=_build output.tex"
