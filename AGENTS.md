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
