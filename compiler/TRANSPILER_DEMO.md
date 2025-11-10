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
fn square(x: i32) -> i32 {
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

## Type System

### Rust-like Defaults
Following Rust's conventions:
- Integer literals default to `i32` (not i64)
- Float literals default to `f64`
- Backward compatible: `int` → `i32`, `float` → `f64`

### Complete Type Mapping

| W Type | Rust Type | Description |
|--------|-----------|-------------|
| **Signed Integers** | | |
| `Int8` | `i8` | 8-bit signed integer |
| `Int16` | `i16` | 16-bit signed integer |
| `Int32` / `int` | `i32` | 32-bit signed (default) |
| `Int64` | `i64` | 64-bit signed integer |
| `Int128` | `i128` | 128-bit signed integer |
| `Int` | `isize` | Pointer-sized signed |
| **Unsigned Integers** | | |
| `UInt8` | `u8` | 8-bit unsigned integer |
| `UInt16` | `u16` | 16-bit unsigned integer |
| `UInt32` | `u32` | 32-bit unsigned integer |
| `UInt64` | `u64` | 64-bit unsigned integer |
| `UInt128` | `u128` | 128-bit unsigned integer |
| `UInt` | `usize` | Pointer-sized unsigned |
| **Floating Point** | | |
| `Float32` | `f32` | 32-bit float |
| `Float64` / `float` | `f64` | 64-bit float (default) |
| **Other Primitives** | | |
| `Bool` / `bool` | `bool` | Boolean |
| `Char` / `char` | `char` | Unicode scalar |
| `String` / `string` | `String` | Owned string |
| **Container Types** | | |
| `List[T]` | `Vec<T>` | Dynamic array |
| `Array[T, N]` | `[T; N]` | Fixed-size array |
| `Slice[T]` | `&[T]` | Slice reference |
| `Map[K,V]` | `HashMap<K, V>` | Hash map |
| `HashSet[T]` | `HashSet<T>` | Hash set |
| `BTreeMap[K,V]` | `BTreeMap<K, V>` | Sorted map |
| `BTreeSet[T]` | `BTreeSet<T>` | Sorted set |

### Examples

**Primitive Types:**
```wolfram
AddBytes[a: UInt8, b: UInt8] := a + b
BigNum[x: Int64] := x * 2
Precision[x: Float32] := x + 1.5
```

**Container Types:**
```wolfram
ProcessList[items: List[Int32]] := items           (* Vec<i32> *)
FixedBuffer[arr: Array[UInt8, 256]] := arr        (* [u8; 256] *)
ReadSlice[data: Slice[UInt8]] := data             (* &[u8] *)
UniqueWords[words: HashSet[String]] := words      (* HashSet<String> *)
SortedIndex[idx: BTreeMap[Int32, String]] := idx  (* BTreeMap<i32, String> *)
OrderedSet[nums: BTreeSet[Int64]] := nums         (* BTreeSet<i64> *)
```

**Backward Compatible (lowercase):**
```wolfram
Square[x: int] := x * x    (* int → i32 *)
Average[x: float] := x / 2  (* float → f64 *)
```

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
