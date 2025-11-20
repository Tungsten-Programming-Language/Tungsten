(* ============================================ *)
(* Tuple Types - Comprehensive Demonstration  *)
(* ============================================ *)
(* Tuples are heterogeneous, fixed-size composite types *)
(* that can hold values of different types together.    *)

(* Basic Tuple Syntax *)
(* ================== *)

(* Two-element tuple using parentheses *)
(1, "hello")

(* Three-element tuple with different types *)
(42, "world", true)

(* Empty tuple (unit type) *)
()

(* Single element tuple - requires trailing comma *)
(42,)

(* Advanced Usage *)
(* ============== *)

(* Nested tuples *)
((1, 2), (3, 4))

(* Tuple with expressions *)
(1 + 2, 3 * 4, 5 ^ 2)

(* Mixed: tuple containing list *)
([1, 2, 3], "list inside tuple")

(* Explicit Tuple constructor syntax *)
Tuple[10, "test"]

(* Function with Tuple Type Annotations *)
(* ==================================== *)

(* Function that returns a tuple *)
MakePair[x: Int32, y: String] := (x, y)

(* Function that takes a tuple as parameter *)
GetFirst[pair: Tuple[Int32, String]] := pair

(* Output Examples *)
(* =============== *)

Print["Two-element tuple:", (100, 200)]
Print["Three-element tuple:", (42, "answer", true)]
Print["Nested tuples:", ((1, 2), (3, 4))]
