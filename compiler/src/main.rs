mod ast;
mod lexer;
mod parser;
mod rust_codegen;

use std::fs;
use std::fs::File;
use std::io::Write;
use std::process::Command;

fn main() {
    // Use command-line argument for input file
    let args: Vec<String> = std::env::args().collect();
    
    // Check if an input file is provided
    let input_file = if args.len() > 1 {
        &args[1]
    } else {
        "hello_world.w"  // Default file
    };

    // Read the contents of the file
    let input = match fs::read_to_string(input_file) {
        Ok(contents) => contents,
        Err(e) => {
            eprintln!("Error reading file {}: {}", input_file, e);
            std::process::exit(1);
        }
    };
    
    let mut parser = parser::Parser::new(input);
    let expr = parser.parse().expect("Failed to parse expression");

    // Use Rust code generation instead of assembly
    let mut rust_codegen = rust_codegen::RustCodeGenerator::new();
    let rust_code = rust_codegen.generate(&expr).expect("Failed to generate Rust code");
    
    // Write Rust code to file
    let output_file = "generated.rs";
    let mut file = File::create(output_file).expect("Failed to create file");
    file.write_all(rust_code.as_bytes()).expect("Failed to write to file");
    
    // Compile the generated Rust code
    let rustc_status = Command::new("rustc")
        .args(&[output_file, "-o", "output"])
        .status()
        .expect("Failed to run rustc");
    
    if !rustc_status.success() {
        eprintln!("Rust compiler (rustc) failed");
        std::process::exit(1);
    }
    
    println!("Compilation of {} complete. Run ./output to see the result.", input_file);
}
