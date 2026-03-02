(* Error propagation with the ? operator *)

(* Basic ? on a Result value *)
GetValue[x: Int32] := Ok[x * 2]

(* ? on Option types *)
FindItem[x: Int32] := Some[x + 1]

(* Chaining multiple ? operations *)
Process[x: Int32] := GetValue[x]?

(* ? combined with the pipe operator *)
Pipeline[x: Int32] := GetValue[x]? |> FindItem?
