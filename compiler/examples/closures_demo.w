(* ============================================ *)
(* Closures and Higher-Order Functions Demo    *)
(* ============================================ *)

(* Map - Apply function to each element *)
Print["Double:", Map[Function[{x}, x * 2], [1, 2, 3]]]

(* Filter - Select elements matching predicate *)
Print["Filter:", Filter[Function[{x}, x > 5], [1, 10, 3]]]

(* Fold - Reduce list to single value *)
Print["Sum:", Fold[Function[{acc, x}, acc + x], 0, [1, 2, 3]]]
