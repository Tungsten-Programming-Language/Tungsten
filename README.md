# The W (Tungsten) Programming Language

## Language Philosophy

This project aims to create a statically-typed, compile-time functional programming language with a syntax inspired by Wolfram Language. The core principles are:

- **Functional Paradigm**: Every computation is an expression
- **Strong Static Typing**: All types determined at compile-time
- **Type Inference**: Reduce verbosity while maintaining type safety
- **Single Binary Compilation**: Efficient, standalone executables
- **Rust-Level Performance**: Compile to native code with minimal overhead

## Language Syntax

The language uses a function-call-based syntax where every operation is a function call:

### Basic Arithmetic
```
Add[1, 2, 3]       # Returns 6
Subtract[10, 5]    # Returns 5
Multiply[2, 3, 4]  # Returns 24
Power[2, 3]        # Returns 8
```

### Function Definition
```
f[x, y] := Power[x, y]
g[x: int, y: int] := Add[x, y]
```

### Data Structures
```
# Lists
[1, 2, 3]                  # List of integers
List[1, 2, 3]              # Equivalent to above
List[1.0, 2.0, 3.0]        # List of floats

# Maps
{a: 1, b: 2, c: 3}         # Map of string keys to integers
```

### Type System
- Primitive Types: `int`, `float`, `string`, `bool`
- Complex Types: `List[type]`, `Map[key_type, value_type]`

## Compilation Goals

1. Compile-time type checking
2. No runtime type overhead
3. Generate efficient native code
4. Minimal runtime dependencies

## Current Status

This is an experimental compiler written in Rust, exploring functional language design and efficient compilation techniques.
