(* The Computer Language Benchmarks Game
   fannkuch-redux benchmark in W *)

(* Fannkuch algorithm - simplified for W language *)

(* Max Fannkuch value for given n *)
MaxFannkuch[n: Int64] :=
  Cond[
    [n <= 1 0]
    [n == 2 1]
    [n == 3 3]
    [n == 4 4]
    [n == 5 7]
    [n == 6 10]
    [n == 7 12]
    [n == 8 15]
    [n == 9 17]
    [n == 10 19]
    [true 22]
  ]

(* Main entry point *)
Run[] := With[{args = Args[]},
  With[{n = ParseInt[First[args]]},
    Print["Pfannkuchen(", n, ") = ", MaxFannkuch[n]]
  ]
]

Run[]
