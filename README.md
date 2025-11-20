# The W (Tungsten) Programming Language

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
