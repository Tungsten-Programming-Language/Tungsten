mod ast;
mod lexer;
mod parser;
mod codegen;

use std::fs;
// Removed unused import
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
    let expr = parser.parse_expression().expect("Failed to parse expression");
    
    let mut codegen = codegen::CodeGenerator::new();
    let assembly = codegen.generate(&expr);
    
    // Write assembly to file
    let mut file = File::create("output.asm").expect("Failed to create file");
    file.write_all(assembly.as_bytes()).expect("Failed to write to file");
    
    // Print generated assembly for debugging
    println!("Generated Assembly:\n{}", assembly);
    
    // Assemble and link (requires nasm and ld)
    let nasm_status = Command::new("nasm")
        .args(&["-f", "elf64", "output.asm"])
        .status()
        .expect("Failed to run nasm");
    
    if !nasm_status.success() {
        eprintln!("Assembler (nasm) failed");
        std::process::exit(1);
    }
        
    let ld_status = Command::new("ld")
        .args(&["output.o", "-o", "output"])
        .status()
        .expect("Failed to run linker");
    
    if !ld_status.success() {
        eprintln!("Linker (ld) failed");
        std::process::exit(1);
    }
    
    println!("Compilation of {} complete. Run ./output to see the result.", input_file);
}
