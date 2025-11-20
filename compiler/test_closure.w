(* Test closures *)
Print["Map test:", Map[Function[{x}, x * 2], [1, 2, 3]]]
Print["Filter test:", Filter[Function[{x}, x > 5], [1, 10, 3, 8]]]
Print["Fold test:", Fold[Function[{acc, x}, acc + x], 0, [1, 2, 3, 4, 5]]]
