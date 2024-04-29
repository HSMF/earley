open Printf
include Fun

let sp = sprintf
let sl map sep lst = lst |> List.map map |> String.concat sep
let ( >>= ) = Option.bind
let ( $> ) op f = Option.map f op
let ( >>> ) f g x = x |> f |> g

module Set = struct
  type 'a t = 'a list

  let rec union xs = function
    | [] -> xs
    | hd :: tl ->
      if List.find_opt (fun x -> x = hd) xs |> Option.is_some
      then union xs tl
      else union (hd :: xs) tl


  let from_list lst =
    let rec helper acc = function
      | [] -> acc
      | hd :: tl -> helper (union acc [ hd ]) tl
    in
    helper [] lst
end

module List = struct
  include List

  let group (n : int) (lst : 'a list) =
    let rec aux acc m lst =
      match lst, acc with
      | [], [] -> []
      | [], x -> [ rev x ]
      | hd :: tl, x ->
        if m = 0 then rev x :: aux [] n (hd :: tl) else aux (hd :: x) (m - 1) tl
    in
    aux [] n lst


  let inspect f lst =
    List.iter f lst;
    lst


  (** takes the first n elements of a list, dropping the rest
      if the list is shorter than n elements, returns the original list

      {[
        List.take 3 [1; 2; 3; 4; 5; 6]] = [1; 2; 3
      ]} *)
  let rec take n lst =
    match n, lst with
    | n, _ when n <= 0 -> []
    | n, x :: xs -> x :: take (n - 1) xs
    | _, [] -> []


  (** drops the first n elements of a list, yielding the rest
      if the list is shorter than n elements, returns []

      {[
        List.skip 3 [1; 2; 3; 4; 5; 6]] = [4; 5; 6
      ]} *)
  and skip n lst =
    match n, lst with
    | 0, x -> x
    | _, [] -> []
    | n, _ :: xs -> skip (n - 1) xs


  let drop = skip

  (** [List.last] [1; 2; 3] = 3 *)
  let last lst = lst |> List.rev |> List.hd
end

let sl_grouped n map sep sep_line lst =
  lst |> List.group n |> List.map (sl map sep) |> sl id sep_line


module Option = struct
  include Option

  let unwrap_or default = fold ~none:default ~some:id

  (** fish operator *)
  let ( ><> ) opt or_else =
    match opt with
    | Some x -> Some x
    | None -> or_else ()


  let unwrap ?(exn = Failure "Unwrapped a None value") = function
    | None -> raise exn
    | Some x -> x
end
