# W Language Transpiler

A transpiler that converts W language (Wolfram-like syntax) to idiomatic Rust code, leveraging Rust's compilation process for safety and performance.

## Implementation Strategy

Following the approach outlined in `REVISED_STRATEGY.md`, this transpiler:
- Converts W code directly to Rust source code
- Maps W types to Rust stdlib types (`List` → `Vec`, `Map` → `HashMap`)
- Inherits all of Rust's safety guarantees through rustc
- Generates idiomatic Rust code with proper naming conventions

## Features Implemented

### 1. Function Definitions
W syntax with type annotations transpiles to Rust functions:

**W Code:**
```wolfram
Square[x: int] := x * x
```

**Generated Rust:**
```rust
fn square(x: i64) -> i64 {
    (x * x)
}
```

### 2. Function Calls
Built-in `Print` function maps to Rust's `println!`:

**W Code:**
```wolfram
Print["Hello, World!"]
```

**Generated Rust:**
```rust
fn main() {
    println!("{}", "Hello, World!".to_string());
}
```

### 3. Lists → Vec
Lists automatically convert to Rust vectors:

**W Code:**
```wolfram
Print[[1, 2, 3, 4, 5]]
```

**Generated Rust:**
```rust
fn main() {
    println!("{:?}", vec![1, 2, 3, 4, 5]);
}
```

### 4. Arithmetic Operations
Binary operations with proper precedence:

**W Code:**
```wolfram
Print[2 + 3 * 4]
```

**Generated Rust:**
```rust
fn main() {
    println!("{}", ((2 + 3) * 4));
}
```

### 5. Multiple Arguments
Functions with multiple arguments:

**W Code:**
```wolfram
Print["The", "answer", "is", 42]
```

**Generated Rust:**
```rust
fn main() {
    println!("{} {} {} {}", "The".to_string(), "answer".to_string(), "is".to_string(), 42);
}
```

### 6. Nested Function Calls
Functions can be composed:

**W Code:**
```wolfram
Print[Square[5]]
```

**Generated Rust:**
```rust
fn main() {
    println!("{}", square(5));
}
```

## Type Mapping

| W Type | Rust Type |
|--------|-----------|
| `int` | `i64` |
| `float` | `f64` |
| `string` | `String` |
| `bool` | `bool` |
| `List[T]` | `Vec<T>` |
| `Map[K,V]` | `HashMap<K, V>` |

## Naming Conventions

The transpiler follows Rust conventions:
- PascalCase function names → snake_case (e.g., `Square` → `square`)
- Type annotations preserved and mapped to Rust types

## Usage

```bash
cargo build
./target/debug/w <input_file.w>
# Generates generated.rs and compiles to ./output
./output
```

## Examples

See the `examples/` directory for more demonstrations:
- `hello_world.w` - Basic printing
- `arithmetic.w` - Mathematical operations
- `list_example.w` - List/Vec usage
- `function_def.w` - Function definitions
- `multiple_args.w` - Multiple function arguments

## Architecture

1. **Lexer** (`lexer.rs`): Tokenizes W source code
2. **Parser** (`parser.rs`): Builds AST from tokens with lookahead for disambiguation
3. **Code Generator** (`rust_codegen.rs`): Translates AST to Rust source
4. **Compiler** (`main.rs`): Coordinates the pipeline and invokes `rustc`

## Key Implementation Details

- **Parser lookahead**: Uses `peek_token()` to distinguish function calls from identifiers
- **Type inference**: Infers return types from function bodies
- **Debug formatting**: Automatically uses `{:?}` for types without `Display` trait
- **Expression vs Statement**: Properly generates Rust expressions without trailing semicolons

## Future Enhancements

Potential additions following the REVISED_STRATEGY.md vision:
- Pattern matching (`Match` expressions)
- Module system
- Ownership annotations (borrows, moves)
- Iterator/map operations
- Result/Option types for error handling
- Async/await support
