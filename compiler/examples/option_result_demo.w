(* Demonstration of Option and Result types in W language *)

(* Basic Option usage - Some values work standalone *)
Some[42]
Some["Hello, World!"]

(* Nested Options *)
Some[Some[100]]

(* Combining with other expressions *)
Some[1 + 2 + 3]

(* Note: None, Ok, and Err require type context in Rust *)
(* They are typically used in function return values or *)
(* in contexts where Rust can infer the full type *)

(* Example showing the syntax: *)
(* None - for empty Option *)
(* Ok[value] - for successful Result *)
(* Err[error] - for error Result *)
