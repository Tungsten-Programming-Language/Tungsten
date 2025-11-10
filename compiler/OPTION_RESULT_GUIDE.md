# Option and Result Types in W

W now supports Rust's Option and Result types for safe error handling. These are crucial for Rust's safety model.

## Syntax

### Option Type
```w
None              (* Empty option - requires type context *)
Some[value]       (* Present option with value *)
```

### Result Type
```w
Ok[value]         (* Success result - requires error type context *)
Err[error]        (* Error result - requires success type context *)
```

## Type Definitions

In W's type system, you can define:
- `Option[T]` - Optional value that can be `None` or `Some[value]`
- `Result[T, E]` - Result that can be `Ok[value]` or `Err[error]`

## Code Generation

W expressions transpile to Rust as follows:

| W Expression | Rust Output |
|-------------|-------------|
| `None` | `None` |
| `Some[42]` | `Some(42)` |
| `Ok[100]` | `Ok(100)` |
| `Err["error"]` | `Err("error".to_string())` |

## Examples

### Basic Usage
```w
(* Option values *)
Some[42]                    (* => Some(42) *)
Some["Hello"]               (* => Some("Hello".to_string()) *)
Some[Some[100]]             (* => Some(Some(100)) *)

(* With expressions *)
Some[1 + 2 + 3]            (* => Some(((1 + 2) + 3)) *)
```

### Important Notes

1. **Type Context Required**: `None`, `Ok`, and `Err` require type context in Rust:
   - `None` needs to know the type it contains
   - `Ok` needs to know the error type
   - `Err` needs to know the success type

2. **Standalone Usage**: These work standalone:
   - ✅ `Some[value]` - type can be inferred from value
   - ❌ `None` - needs explicit type annotation in Rust
   - ❌ `Ok[value]` - needs error type
   - ❌ `Err[error]` - needs success type

3. **Best Practices**:
   - Use within function return types where types can be inferred
   - Use in match expressions where the full type is known
   - Prefer explicit function signatures when using Result types

## Type System Integration

W's type system includes:
```
Type::Option(Box<Type>)                  (* Option<T> *)
Type::Result(Box<Type>, Box<Type>)       (* Result<T, E> *)
```

These transpile directly to Rust's standard library types without any additional dependencies.

## Testing

Run the test suite to verify Option/Result functionality:
```bash
cargo test option_result
```

This runs 13 comprehensive tests covering:
- Lexer token recognition
- Parser expression creation
- Code generation
- Nested structures
