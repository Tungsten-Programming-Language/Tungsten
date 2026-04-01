# Tungsten (W) Programming Language

A functional programming language that transpiles to Rust with Wolfram Language-inspired syntax.

## Syntax Overview

- **Function calls**: `FunctionName[arg1, arg2]`
- **Operators**: `1 + 2`, `x * x`
- **Comments**: `# comment` or `(* comment *)`
- **Variables**: Immutable bindings via `With[{x = 5}, body]`
- **Functions**: `f[x, y] := expression` or with types `f[x: Int32] := x * x`
- **Lambdas**: `Function[{x}, x * 2]` or shorthand `x -> x * 2`
- **Conditionals**: `Cond[[cond1, val1], [cond2, val2], [default]]`
- **Pattern matching**: `Match[value, [Some[x], x], [None, 0]]`
- **Loops**: `Do[body, {i, n}]`, `While[cond, body]`

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
