# The W (Tungsten) Programming Language

## What is the Tungsten programming language?

Tungsten (W) is a functional programming language that transpiles to Rust, heavily influenced by the syntax of the Wolfram Language used in Mathematica. In its initial phase, its primary focus is on data engineering tasks but could be extended to all areas with additional library packages.

## Motivation

Rust offers the best overall memory and processing performance, memory safety, security, and verifiability among mainstream programming languages today. However, its memory safety rules often create verbose, nested code structures that are difficult to read and write. When syntax becomes complex, the focus is shifted from the procedures, which should be the focus, to syntactical correctness. What Tungsten aims to achieve is a whole new approach to writing programs that still offers the benefits of the Rust compiler.

## Language Approach

Since LISP was developed at MIT, there has been a number of small but dedicated communities around LISP-like languages such as Clojure who extoll the virtues of a lambda calculus-based syntax. One of the major benefits is language simplicity. However, these languages have not been widely adopted, in the author's opinion, because of the inside-out parsing (onion parsing) of the source code. With many nested parenthetical groups, it can be very difficult to read with a bias towards C-like languages.

Wolfram Language meets this half way by using a syntax where functions take bracketed arguments but still reads from the outside in. This change yields significantly easier to read source code that is still concise and functional.

### Syntax Comparison

Consider a function that computes the average of the squares of numbers in a filtered list. Here's how it looks in both LISP and Wolfram Language:

**LISP (inside-out parsing):**
```lisp
(/ (reduce #'+ 
           (mapcar (lambda (x) (* x x))
                   (remove-if-not (lambda (x) (> x 5)) 
                                  '(1 6 3 8 2 9 4 7))))
   (length (remove-if-not (lambda (x) (> x 5)) 
                          '(1 6 3 8 2 9 4 7))))
```

**Wolfram Language (outside-in parsing):**
```wolfram
Divide[
  Total[
    Map[
      Function[x, Times[x, x]],
      Select[Range[1, 6, 3, 8, 2, 9, 4, 7], Function[x, Greater[x, 5]]]
    ]
  ],
  Length[
    Select[Range[1, 6, 3, 8, 2, 9, 4, 7], Function[x, Greater[x, 5]]]
  ]
]
```

In the LISP version, you must parse from the innermost parentheses outward, making it difficult to understand the overall structure at a glance. The Wolfram Language version reads naturally from outside-in: "Divide the total of mapped squares by the length of the selection." Tungsten adopts this readable, functional syntax while transpiling to high-performance Rust code.

## What Tungsten Offers

Tungsten offers a language that hides much of the complexity of the Rust borrow checker, mutability rules, and verbosity from the users so that they can focus on the problem to be solved with a cognitive ease comparable to Python and Ruby. Its parser does its own type checking to ensure that the transpiled Rust code is compilable, then Rust takes over and compiles the executable binary. The end product is an executable that offers all the performance and safety guarantees of Rust.

## Language Philosophy

W is a statically-typed functional programming language with a syntax inspired by Wolfram Language that transpiles to Rust. The core principles are:

- **Functional Paradigm**: Every computation is an expression
- **Strong Static Typing**: All types determined at compile-time
- **Type Inference**: Reduce verbosity while maintaining type safety
- **Transpiles to Rust**: Leverages Rust's performance, safety, and ecosystem
- **Rust-Level Performance**: Generated Rust code compiles to efficient native binaries

## Language Syntax

The language uses a function-call-based syntax where every operation is a function call:

### Hello World
```
Print["Hello, World!"]
```

### Basic Arithmetic
```
Add[1, 2, 3]       # Returns 6
Subtract[10, 5]    # Returns 5
Multiply[2, 3, 4]  # Returns 24
Divide[10, 2]      # Returns 5
Power[2, 3]        # Returns 8

(* Or use infix operators *)
1 + 2 + 3          # Returns 6
x * x              # Squaring
```

### Function Definition
```
(* Without type annotations *)
f[x, y] := Power[x, y]

(* With type annotations *)
Square[x: Int32] := x * x

(* Using the function *)
Print[Square[5]]   # Outputs: 25

(* Multiple parameters with types *)
Add[x: Int32, y: Int32] := x + y
```

### Conditionals
```
(* Cond expression - similar to LISP's cond *)
Cond[
  [condition1, statements1],
  [condition2, statements2],
  [default_statements]
]
```

### Data Structures
```
(* Lists - transpiles to Vec<T> in Rust *)
[1, 2, 3]                        # List of integers
List[1, 2, 3]                    # Equivalent
ProcessList[items: List[Int32]] := items

(* Arrays - fixed size *)
Array[Int32, 5]                  # Fixed-size array of 5 Int32s

(* Slices - borrowed views *)
Slice[Int32]                     # Slice of Int32s

(* HashSets - unique elements *)
UniqueItems[items: HashSet[String]] := items

(* Maps *)
Map[String, Int32]               # HashMap in Rust
BTreeMap[String, Int32]          # Ordered map
BTreeSet[Int32]                  # Ordered set
```

### Tuples
```
(* Tuples - heterogeneous, fixed-size composite types *)
(1, "hello")                     # Two-element tuple
(42, "answer", true)             # Three-element tuple with different types
()                               # Empty tuple (unit type)
(42,)                            # Single-element tuple (note trailing comma)

(* Nested tuples *)
((1, 2), (3, 4))                 # Tuple of tuples

(* Explicit constructor syntax *)
Tuple[10, "test"]                # Alternative syntax

(* In function signatures *)
MakePair[x: Int32, y: String] := (x, y)
GetFirst[pair: Tuple[Int32, String]] := pair
```

### Option and Result Types
```
(* Option types - for nullable values *)
Some[42]                         # Some value
Some["Hello, World!"]
Some[Some[100]]                  # Nested options
None                             # Empty option

(* Result types - for error handling *)
Ok[value]                        # Success case
Err[error]                       # Error case
```

### Pattern Matching
```
(* Match expression - destructure and match values *)
Match[value,
  [pattern1, result1],
  [pattern2, result2],
  [pattern3, result3]
]

(* Wildcard pattern - matches anything *)
Match[x, [_, "default"]]

(* Literal patterns *)
Match[5,
  [1, "one"],
  [2, "two"],
  [_, "other"]
]

(* Variable binding *)
Match[100, [x, x]]               # Binds value to x

(* Option patterns *)
Match[Some[42],
  [Some[x], x],
  [None, 0]
]

(* Tuple patterns *)
Match[(1, 2),
  [(x, y), x]                    # Destructure tuple
]

(* Nested patterns *)
Match[Some[(42, "answer")],
  [Some[(num, str)], num],
  [None, 0]
]
```

### Local Bindings with `With`

W provides the `With` expression for creating local immutable bindings in a lexical scope. This is inspired by Wolfram Language and is particularly useful for data pipelines where intermediate values need names.

```
(* Simple binding *)
With[{x = 5}, x + 1]             (* Returns 6 *)

(* Multiple bindings *)
With[{x = 1, y = 2}, x + y]      (* Returns 3 *)

(* Sequential bindings - later bindings can reference earlier ones *)
With[{x = 1, y = x + 1}, y * 2]  (* Returns 4 *)

(* Bindings are immutable - shadowing creates new scope *)
With[{x = 1}, With[{x = 2}, x]]  (* Inner x shadows outer x *)

(* Useful in data pipelines *)
data |> Filter[x -> x > 0] |> With[{
  positive_data = _
}, Map[y -> y * 2, positive_data]]

(* With comprehensible intermediate steps *)
With[{
  raw_data = ReadCSV["data.csv"],
  cleaned = Filter[row -> row.active == true, raw_data],
  transformed = Map[row -> row.salary * 1.1, cleaned]
}, transformed]
```

Bindings are **sequential** - each binding can reference variables from previous bindings in the same `With` expression. The body expression has access to all bindings in its scope. Bindings are **immutable** - they cannot be reassigned (use `Module` for mutable state, coming soon).

### Loops and Iteration

W provides both procedural loops and functional iteration constructs, inspired by Wolfram Language.

#### Do Loop

The `Do` loop iterates a fixed number of times:

```
(* Simple iteration - execute body 3 times *)
Do[Print["Hello"], 3]

(* With loop variable - i goes from 1 to 5 *)
Do[Print[i], {i, 5}]

(* With range - i goes from 2 to 5 *)
Do[Print[i], {i, 2, 5}]

(* With step - i goes from 1 to 10 by 2 *)
Do[Print[i], {i, 1, 10, 2}]

(* Nested loops *)
Do[Do[Print[i + j], {j, 3}], {i, 2}]
```

#### While Loop

The `While` loop iterates while a condition is true:

```
(* Simple while loop *)
While[condition, body]

(* Example with Break *)
Do[
  If[i == 5, Break[]],
  {i, 10}
]
```

#### Break and Continue

Control flow inside loops:

```
Break[]      (* Exit the loop immediately *)
Continue[]   (* Skip to next iteration *)
```

### Closures and Higher-Order Functions
```
(* Lambda/Anonymous function syntax *)
Function[{x}, x * 2]             # Single parameter
Function[{x, y}, x + y]          # Multiple parameters
Function[{x: Int32}, x * x]      # With type annotation

(* Map - transform each element *)
Map[Function[{x}, x * 2], [1, 2, 3]]
(* Result: [2, 4, 6] *)

(* Filter - select elements *)
Filter[Function[{x}, x > 5], [1, 10, 3, 8]]
(* Result: [10, 8] *)

(* Fold - reduce to single value *)
Fold[Function[{acc, x}, acc + x], 0, [1, 2, 3, 4, 5]]
(* Result: 15 *)

(* Nested operations *)
Map[
  Function[{x}, x * 2],
  Filter[Function[{x}, x > 2], [1, 2, 3, 4, 5]]
]
(* Result: [6, 8, 10] *)
```

### Type System

W supports a comprehensive type system that maps directly to Rust types:

#### Signed Integers
- `Int8`, `Int16`, `Int32`, `Int64`, `Int128`
- `Int` (platform-dependent, equivalent to Rust's `isize`)

#### Unsigned Integers
- `UInt8`, `UInt16`, `UInt32`, `UInt64`, `UInt128`
- `UInt` (platform-dependent, equivalent to Rust's `usize`)

#### Floating Point
- `Float32` (f32 in Rust)
- `Float64` (f64 in Rust)

#### Other Primitives
- `Bool`
- `Char`
- `String`

#### Composite Types
- `Tuple[T1, T2, ...]` - Heterogeneous, fixed-size tuple ((T1, T2, ...) in Rust)
  - Can contain different types
  - Supports nesting: `Tuple[Int32, Tuple[String, Bool]]`
  - Empty tuple `()` represents the unit type

#### Container Types
- `List[T]` - Dynamic array (Vec<T> in Rust)
- `Array[T, N]` - Fixed-size array ([T; N] in Rust)
- `Slice[T]` - Borrowed view into a sequence (&[T] in Rust)
- `Map[K, V]` - Hash map (HashMap<K, V> in Rust)
- `HashSet[T]` - Set of unique values (HashSet<T> in Rust)
- `BTreeMap[K, V]` - Ordered map (BTreeMap<K, V> in Rust)
- `BTreeSet[T]` - Ordered set (BTreeSet<T> in Rust)

#### Error Handling Types
- `Option[T]` - Optional values (Option<T> in Rust)
  - `Some[value]` - Present value
  - `None` - Absent value
- `Result[T, E]` - Result of operations that can fail (Result<T, E> in Rust)
  - `Ok[value]` - Success case
  - `Err[error]` - Error case

#### Function Types
- `Function[arg_types..., return_type]` - Function signatures

## Transpilation Goals

1. **Compile-time type checking**: All type errors caught during transpilation
2. **Zero runtime overhead**: Direct mapping to Rust types with no abstraction penalty
3. **Idiomatic Rust generation**: Produce clean, readable Rust code
4. **Leverage Rust ecosystem**: Access to Rust's safety guarantees and performance
5. **Minimal runtime dependencies**: Generated code relies only on Rust's standard library

## How It Works

1. **Parse**: W source code is parsed into an Abstract Syntax Tree (AST)
2. **Type Check**: Static type analysis ensures type safety
3. **Transpile**: AST is transformed into equivalent Rust code
4. **Compile**: Generated Rust code is compiled by `rustc` into a native binary

## Current Status

This is an experimental transpiler written in Rust, exploring functional language design and Rust code generation. The project demonstrates how a high-level functional syntax can compile down to efficient, safe Rust code.

---
Written with [Z.AI](https://z.ai) in [Opencode](https://opencode.ai)
