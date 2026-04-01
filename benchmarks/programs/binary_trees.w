(* The Computer Language Benchmarks Game
   binary-trees benchmark in W *)

(* Count nodes in a conceptual tree of given depth *)
(* A full binary tree of depth n has 2^(n+1) - 1 nodes *)
CountNodes[depth: Int64] := 
  Cond[
    [depth <= 0 1]
    [true 1 + 2 * CountNodes[depth - 1]]
  ]

(* Main entry point *)
Run[] := With[{args = Args[]},
  With[{n = ParseInt[First[args]]},
    With[{minDepth = 4},
      With[{maxDepth = Cond[
        [minDepth + 2 > n minDepth + 2]
        [true n]
      ]},
        With[{stretchDepth = maxDepth + 1},
          Print["stretch tree of depth ", stretchDepth, " has ", CountNodes[stretchDepth], " nodes"]
        ]
      ]
    ]
  ]
]

Run[]
