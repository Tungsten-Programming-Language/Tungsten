mod ast;
mod lexer;
mod parser;
mod codegen;

use std::fs::File;
use std::io::Write;
use std::process::Command;

fn main() {
    let input = "2 + 3 * 4".to_string();
    
    let mut parser = parser::Parser::new(input);
    let expr = parser.parse_expression().expect("Failed to parse expression");
    
    let mut codegen = codegen::CodeGenerator::new();
    let assembly = codegen.generate(&expr);
    
    // Write assembly to file
    let mut file = File::create("output.asm").expect("Failed to create file");
    file.write_all(assembly.as_bytes()).expect("Failed to write to file");
    
    // Assemble and link (requires nasm and ld)
    Command::new("nasm")
        .args(&["-f", "elf64", "output.asm"])
        .status()
        .expect("Failed to assemble");
        
    Command::new("ld")
        .args(&["output.o", "-o", "output"])
        .status()
        .expect("Failed to link");
}
