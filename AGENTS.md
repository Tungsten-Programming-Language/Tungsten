# Tungsten (W) Programming Language

A functional programming language that transpiles to Rust with Wolfram Language-inspired syntax.

## Syntax Overview

- **Function calls**: `FunctionName[arg1, arg2]`
- **Operators**: `1 + 2`, `x * x`, `2 ^ 3` (power)
- **Comments**: `# comment` or `(* comment *)`
- **Variables**: Immutable bindings via `With[{x = 5}, body]`
- **Functions**: `f[x, y] := expression` or with types `f[x: Int32] := x * x`
- **Lambdas**: `Function[{x}, x * 2]` or shorthand `x -> x * 2`
- **Pipe operator**: `data |> Map[x -> x * 2] |> Filter[x -> x > 5]`
- **Error propagation**: `GetValue[x]?` (unwraps Option/Result, short-circuits on None/Err)
- **Conditionals**: `If[cond, then_expr]` or `Cond[[cond1, val1], [cond2, val2], [default]]`
- **Pattern matching**: `Match[value, [Some[x], x], [None, 0]]`
- **Structs**: `Struct[Point, [x: Int32, y: Int32]]`
- **Loops**: `Do[body, {i, n}]`, `While[cond, body]`, `Break[]`, `Continue[]`
- **Higher-order functions**: `Map[f, list]`, `Filter[f, list]`, `Fold[f, init, list]`, `FlatMap[f, list]`, `Take[n, list]`, `Zip[list1, list2]`, `GroupBy[key_fn, list]`
- **List operations**: `Append[list, item]`, `Length[list]`, `First[list]`, `Rest[list]`, `Reverse[list]`, `Concat[a, b]`, `Nth[list, i]`
- **Input/Output**: `ReadLine[]`, `Print[msg]`, `Args[]`
- **Lazy iterators**: `LazyMap[f, list]`, `LazyFilter[f, list]`, `Collect[iter]`
- **Logging**: `LogDebug[msg]`, `LogInfo[msg]`, `LogWarn[msg]`, `LogError[msg]`

## Primitive Operators

### Arithmetic
| Operator | Syntax | Description |
|----------|--------|-------------|
| `+` | `a + b` | Addition |
| `-` | `a - b` | Subtraction |
| `*` | `a * b` | Multiplication |
| `/` | `a / b` | Division |
| `^` | `a ^ b` | Power (exponentiation) |

### Comparison
| Operator | Syntax | Description |
|----------|--------|-------------|
| `==` | `a == b` | Equality |
| `!=` | `a != b` | Inequality |
| `<` | `a < b` | Less than |
| `>` | `a > b` | Greater than |
| `<=` | `a <= b` | Less than or equal |
| `>=` | `a >= b` | Greater than or equal |

### Logical
| Operator | Syntax | Description |
|----------|--------|-------------|
| `&&` | `a && b` | Logical AND (short-circuit) |
| `\|\|` | `a \|\| b` | Logical OR (short-circuit) |
| `!` | `!a` | Logical NOT |

### Special
| Operator | Syntax | Description |
|----------|--------|-------------|
| `\|>` | `x \|> f` | Pipe (passes left as last arg to right) |
| `->` | `x -> body` | Lambda shorthand |
| `?` | `expr?` | Error propagation (unwrap Option/Result) |

### Operator Precedence (lowest to highest)
1. `\|>` (pipe, left-associative)
2. `&&` (logical AND)
3. `\|\|` (logical OR)
4. `==`, `!=`, `<`, `>`, `<=`, `>=` (comparison)
5. `+`, `-` (additive)
6. `*`, `/` (multiplicative)
7. `^` (power, right-associative)
8. `?` (propagate, unary postfix)
9. `!` (logical NOT, unary prefix)
10. `-` (unary minus)

## Types

- **Primitives**: `Int32`, `Float64`, `Bool`, `String`, `Char`
- **Collections**: `List[T]`, `Array[T, N]`, `Map[K, V]`, `HashSet[T]`
- **Option/Result**: `Option[T]` (`Some[v]`/`None`), `Result[T, E]` (`Ok[v]`/`Err[e]`)
- **Tuples**: `(1, "hello")`

## Lazy Iterators

Lazy iterators avoid intermediate allocations when chaining operations:

| Function | Description |
|----------|-------------|
| `LazyMap[f, list]` | Returns iterator (no collection) |
| `LazyFilter[pred, list]` | Returns iterator (no collection) |
| `Collect[iter]` | Materializes iterator into List |

**Eager vs Lazy comparison:**
```
(* Eager - creates intermediate Vec after each operation *)
[1, 2, 3] |> Map[x -> x * 2] |> Filter[x -> x > 3]

(* Lazy - single allocation at the end *)
[1, 2, 3] |> LazyMap[x -> x * 2] |> LazyFilter[x -> x > 3] |> Collect[]
```

## Input/Output

| Function | Signature | Description |
|----------|-----------|-------------|
| `ReadLine[]` | `: Option[String]` | Read line from stdin, returns `Some[line]` or `None` on EOF |
| `Print[msg]` | `: ()` | Print message to stdout |
| `Args[]` | `: List[String]` | Get command-line arguments |

**ReadLine usage:**
```
(* Pattern match on result *)
Match[ReadLine[],
    [Some[line], Print["You entered: ", line]],
    [None, Print["EOF reached"]]
]
```

## List Operations

| Function | Signature | Description |
|----------|-----------|-------------|
| `Append[list, item]` | `List[T] -> List[T]` | Append item to list, returns mutated list |
| `Length[list]` | `List[T] -> Int64` | Get list length |
| `First[list]` | `List[T] -> T` | Get first element |
| `Rest[list]` | `List[T] -> List[T]` | Get all elements except first |
| `Reverse[list]` | `List[T] -> List[T]` | Reverse list |
| `Concat[a, b]` | `List[T] -> List[T] -> List[T]` | Concatenate two lists |
| `Nth[list, i]` | `List[T] -> Int64 -> T` | Get element at index |
| `Set[list, i, v]` | `List[T] -> Int64 -> T -> List[T]` | Set element at index |

## Language Restrictions & Known Issues

Verified working (transpiled and executed end-to-end), but the parser and
codegen have these sharp edges:

**Parser restrictions:**

- Statements are separated by newlines only. There is **no semicolon token**
  in the lexer — `stmt1; stmt2` fails to parse.
- `Module[{...}, body]` accepts a **single body expression** only. Multi-
  statement programs must be newline-separated top-level forms.
- **Bare top-level assignments are not supported** (`l = [1, 2, 3]` at file
  top level fails to parse). Bindings must live inside `With[{...}, ...]` or
  `Module[{...}, ...]`.

**Codegen issues:**

- `Print[Match[...]]` where the match arms are `Print[...]` (unit-typed arms)
  fails rustc: the outer `println!` tries to format the match's `()` result.
  Use the bare `Match[ReadLine[], ...]` form at top level instead.
- `StringJoin` exists in codegen but emits an unscoped `string_join` call,
  which does not link. Avoid it until it is scoped into the stdlib.

## Repository Structure

```
.
├── README.md           # Language documentation
├── compiler/           # Rust transpiler
│   ├── src/
│   │   ├── lexer.rs        # Tokenizer
│   │   ├── parser.rs       # Parser → AST
│   │   ├── ast.rs          # AST definitions
│   │   ├── type_checker.rs # Type validation
│   │   ├── type_inference.rs # Type inference
│   │   └── rust_codegen.rs # Rust code generation
│   ├── stdlib/         # Standard library
│   ├── tests/          # Integration tests
│   └── examples/       # Example programs
└── benchmarks/         # Performance benchmarks
```

## Build & Test

```bash
cd compiler && cargo test    # Run tests
cd compiler && cargo run -- <file.w>  # Transpile a file
```
