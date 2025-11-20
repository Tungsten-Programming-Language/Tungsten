Print["Double:", Map[Function[{x}, x * 2], [1, 2, 3]]]
Print["Filter:", Filter[Function[{x}, x > 5], [1, 10, 3]]]
Print["Sum:", Fold[Function[{acc, x}, acc + x], 0, [1, 2, 3]]]
