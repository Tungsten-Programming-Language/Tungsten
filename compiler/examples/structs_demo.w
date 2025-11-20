(* Custom Struct Definitions in W *)

(* Simple struct with two fields *)
Struct[Point, [x: Int32, y: Int32]]

(* Create a point *)
Print["Point:", Point[10, 20]]

(* Struct with different types *)
Struct[Person, [name: String, age: Int32]]

(* Create a person *)
Print["Person:", Person["Alice", 30]]

(* Struct with multiple Int32 fields *)
Struct[Rectangle, [width: Int32, height: Int32, x: Int32, y: Int32]]

(* Create a rectangle *)
Print["Rectangle:", Rectangle[100, 50, 0, 0]]

(* Struct with computed values *)
Print["Computed Point:", Point[5 + 5, 10 * 2]]

(* Another rectangle with calculations *)
Print["Dynamic Rectangle:", Rectangle[10 * 2, 5 + 10, 0, 0]]
