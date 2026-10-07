(* End-to-end test: WriteFile then ReadFile *)

(* Step 1: write a todo file (top-level statements, newline-separated) *)
Match[WriteFile["/tmp/todo_demo.txt", "buy milk\nwalk dog\n"],
    [Ok[()], Print["write: OK"]],
    [Err[msg], Print["write failed: ", msg]]
]

(* Step 2: read it back *)
Match[ReadFile["/tmp/todo_demo.txt"],
    [Some[content], Print["read: [", content, "]"]],
    [None, Print["read failed: file not found"]]
]

(* Step 3: read a file that doesn't exist -> None path *)
Match[ReadFile["/tmp/does_not_exist_12345.txt"],
    [Some[content], Print["unexpected: ", content]],
    [None, Print["missing file: correctly None"]]
]