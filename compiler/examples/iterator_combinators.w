(* ============================================ *)
(* Iterator Combinators Demo                    *)
(* ============================================ *)

(* Take - take first n elements *)
Print["Take 3:", Take[3, [1, 2, 3, 4, 5]]]

(* Zip - combine two lists into tuples *)
Print["Zip:", Zip[[1, 2, 3], ["a", "b", "c"]]]

(* FlatMap - map and flatten results *)
Print["FlatMap:", FlatMap[Function[{x}, [x, x * 2]], [1, 2, 3]]]

(* Using pipe operator with combinators *)
Print["Piped Take:", [10, 20, 30, 40, 50] |> Take[2]]

Print["Piped FlatMap:", [1, 2, 3] |> FlatMap[x -> [x, x * 10]]]

(* GroupBy - group elements by key function *)
Print["GroupBy:", GroupBy[Function[{x}, x / 10], [5, 12, 23, 8, 15]]]

(* Combining combinators *)
Print["Chained:", [1, 2, 3, 4, 5, 6, 7, 8, 9, 10] |> Filter[Function[{x}, x > 3]] |> Take[3] |> Map[Function[{x}, x * x]]]
