(*
  This file demonstrates all container type annotations.
  The function signatures show the W→Rust type mappings.
*)

(* Vec<i32> *)
UseList[items: List[Int32]] := items

(* [i32; 10] - fixed-size array *)
UseArray[buffer: Array[Int32, 10]] := buffer

(* &[u8] - slice reference *)
UseSlice[data: Slice[UInt8]] := data

(* HashMap<String, Int32> *)
UseHashMap[mapping: Map[String, Int32]] := mapping

(* HashSet<String> *)
UseHashSet[unique: HashSet[String]] := unique

(* BTreeMap<Int32, String> - sorted map *)
UseBTreeMap[sorted: BTreeMap[Int32, String]] := sorted

(* BTreeSet<Int64> - sorted set *)
UseBTreeSet[ordered: BTreeSet[Int64]] := ordered
