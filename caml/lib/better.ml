module IntMap = Stdlib.Map.Make (Int)

module Item = struct
  type t = Naive.item
  let compare a b = (failwith "todo compare")
end

module ItemSet = Stdlib.Set.Make (Item)

let next_item_set (j : int) (prev : ItemSet.t IntMap.t) =
  let item = failwith "todo" in
  IntMap.add j item prev
