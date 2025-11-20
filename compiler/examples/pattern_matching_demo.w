(* ============================================ *)
(* Pattern Matching - Comprehensive Demo       *)
(* ============================================ *)
(* Pattern matching allows you to destructure   *)
(* values and execute different code based on   *)
(* the shape and content of the data.           *)

(* Basic Pattern Matching *)
(* ====================== *)

(* Wildcard pattern - matches anything *)
Match[42, [_, "matches anything"]]

(* Literal patterns - match exact values *)
Match[5,
  [1, "one"],
  [2, "two"],
  [3, "three"],
  [_, "other number"]
]

(* Variable binding - binds matched value to variable *)
Match[100, [x, x]]

(* String patterns - use variable binding *)
(* Note: String literal patterns have type matching complexity *)
(* Use variable patterns instead for strings *)
Match["hello",
  [msg, msg]
]

(* Boolean patterns *)
Match[true,
  [true, "yes"],
  [false, "no"]
]

(* Option Type Patterns *)
(* ==================== *)

(* Pattern matching with Some *)
Match[Some[42],
  [Some[x], x],
  [None, 0]
]

(* Nested Option patterns *)
Match[Some[Some[100]],
  [Some[Some[val]], val],
  [Some[None], 0],
  [None, 0]
]

(* Result Type Patterns *)
(* ==================== *)
(* Note: Result type patterns require type annotations in Rust *)
(* For demonstration, using Option types instead *)

(* Pattern matching similar to Result *)
Match[Some[42],
  [Some[value], value],
  [None, 0]
]

(* Tuple Patterns *)
(* ============== *)

(* Destructuring two-element tuples *)
Match[(1, 2),
  [(x, y), x]
]

(* Destructuring three-element tuples *)
Match[(10, "hello", true),
  [(num, str, bool), num]
]

(* Nested tuple patterns *)
Match[((1, 2), (3, 4)),
  [((a, b), (c, d)), a]
]

(* Mixed tuple patterns with Option *)
Match[(Some[5], Some[10]),
  [(Some[x], Some[y]), x],
  [(None, _), 0],
  [(_, None), 0]
]

(* List Patterns *)
(* ============= *)
(* Note: List patterns are currently limited due to Rust's *)
(* Vec vs slice distinction. Use wildcard or variable patterns *)
(* for lists until slice pattern support is fully implemented. *)

(* Use wildcard for list matching *)
Match[[1, 2, 3],
  [_, "any list"]
]

(* Complex Nested Patterns *)
(* ======================= *)

(* Combining multiple pattern types *)
Match[Some[(42, "answer")],
  [Some[(num, str)], num],
  [None, 0]
]

(* Deeply nested patterns *)
Match[Some[Some[(1, 2)]],
  [Some[Some[(x, y)]], x],
  [Some[None], 0],
  [None, 0]
]

(* Pattern Matching in Print Statements *)
(* ==================================== *)

Print["Wildcard match:",
  Match[99, [_, "anything"]]
]

Print["Number match:",
  Match[7,
    [7, "lucky seven"],
    [13, "unlucky thirteen"],
    [_, "some other number"]
  ]
]

Print["Option match:",
  Match[Some[42],
    [Some[x], x],
    [None, 0]
  ]
]

Print["Option match (nested):",
  Match[Some[Some[100]],
    [Some[Some[val]], val],
    [_, 0]
  ]
]

Print["Tuple match:",
  Match[(100, 200),
    [(a, b), a]
  ]
]

(* Practical Examples *)
(* ================== *)

(* Safe computation using Option *)
Match[Some[10],
  [Some[result], result],
  [None, 0]
]

(* Configuration with Options *)
Match[Some["config.json"],
  [Some[filename], filename],
  [None, "default.json"]
]

(* Status codes *)
Match[200,
  [200, "OK"],
  [404, "Not Found"],
  [500, "Internal Server Error"],
  [_, "Unknown Status"]
]
