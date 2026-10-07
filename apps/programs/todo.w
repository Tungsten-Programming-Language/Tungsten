(*
 * Todo - a persistent CLI todo list in W
 *
 * Commands:
 *   list            - show todos with numbers
 *   add <text>      - add a todo
 *   done <n>        - mark todo n as done
 *   quit            - exit
 *
 * Storage: todos.txt (one todo per line, current directory)
 *)

(* Read the todo file, or empty string if missing *)
LoadTodos[] := Match[ReadFile["todos.txt"],
    [Some[content], content],
    [None, ""]
]

(* Drop a trailing blank line produced by a file ending in newline *)
CleanLines[lines: List[String]] := Cond[
    [Length[lines] > 0 && Nth[lines, Length[lines] - 1] == "" Rest[lines]]
    [true lines]
]

(* Show all todos as a numbered list; "(no todos)" when empty *)
ListTodos[] := With[{content = LoadTodos[]},
    With[{lines = CleanLines[StringSplit[content, "\n"]]},
        If[Length[lines] == 0,
            Print["(no todos)"],
            Do[Print[StringJoin[[ToString[i + 1], ". ", Nth[lines, i]], ""]], {i, 0, Length[lines] - 1}]
        ]
    ]
]

(* Append a todo to the file *)
AddTodo[text: String] := If[LoadTodos[] == "",
    WriteFile["todos.txt", text],
    With[{lines = CleanLines[StringSplit[LoadTodos[], "\n"]]},
        WriteFile["todos.txt", StringJoin[Append[lines, text], "\n"]]
    ]
]

(* Mark todo n (1-based) as done by prefixing it with "[x] " *)
DoneTodo[n: Int64] := With[{content = LoadTodos[]},
    If[content == "",
        Print["no todos yet"],
        With[{lines = CleanLines[StringSplit[content, "\n"]]},
            With[{count = ParseInt[ToString[Length[lines]]]},
                Cond[
                    [n < 1 || n > count Print["invalid number: ", ToString[n]]]
                    [true With[{updated = Set[lines, n - 1, StringJoin[["[x] ", Nth[lines, n - 1]], ""]]},
                        WriteFile["todos.txt", StringJoin[updated, "\n"]]
                    ]]
                ]
            ]
        ]
    ]
]

(* Help text printed once at startup *)
Print["[todos] list | add <text> | done <n> | quit"]

(* Main menu loop: the While body is a single expression *)
While[true,
    Match[ReadLine[],
        [Some[cmd],
            Cond[
                [cmd == "quit" Break[]]
                [cmd == "list" ListTodos[]]
                [cmd == "add" Print["usage: add <text>"]]
                [cmd == "done" Print["usage: done <n>"]]
                [Length[StringSplit[cmd, " "]] >= 2 && Nth[StringSplit[cmd, " "], 0] == "add" AddTodo[StringJoin[Rest[StringSplit[cmd, " "]], " "]]]
                [Length[StringSplit[cmd, " "]] >= 2 && Nth[StringSplit[cmd, " "], 0] == "done" DoneTodo[ParseInt[Nth[StringSplit[cmd, " "], 1]]]]
                [true Print["unknown command: ", cmd]]
            ]
        ],
        [None, Break[]]
    ]
]