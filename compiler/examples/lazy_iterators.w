(* ============================================ *)
(* Lazy Iterators Demo                          *)
(* ============================================ *)

(* Eager vs Lazy comparison *)

(* Eager Map - immediately collects results *)
Print["Eager Map:", [1, 2, 3, 4, 5] |> Map[x -> x * 2]]

(* LazyMap - returns iterator, no collection yet *)
(* Use with Collect to materialize the result *)
Print["Lazy Map + Collect:", [1, 2, 3, 4, 5] |> LazyMap[x -> x * 2] |> Collect[]]

(* Eager Filter *)
Print["Eager Filter:", [1, 2, 3, 4, 5] |> Filter[x -> x > 2]]

(* LazyFilter - returns iterator *)
Print["Lazy Filter + Collect:", [1, 2, 3, 4, 5] |> LazyFilter[x -> x > 2] |> Collect[]]

(* Chaining lazy operations *)
(* This avoids intermediate Vec allocations *)
Print["Lazy Chain:", [1, 2, 3, 4, 5, 6, 7, 8, 9, 10] |> LazyMap[x -> x * x] |> LazyFilter[x -> x > 25] |> Collect[]]

(* Multiple lazy operations in a pipeline *)
Print["Multi-stage:", [1, 2, 3, 4, 5, 6, 7, 8, 9, 10] |> LazyFilter[x -> x > 3] |> LazyMap[x -> x * 2] |> LazyFilter[x -> x < 15] |> Collect[]]

(* Benefit: No intermediate collections *)
(* Eager: [1,2,3] |> Map[f] creates Vec, then |> Filter[g] creates another Vec *)
(* Lazy:  [1,2,3] |> LazyMap[f] |> LazyFilter[g] |> Collect[] creates only ONE Vec *)
