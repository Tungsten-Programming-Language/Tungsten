use crate::ast::{Expression, Operator, LogLevel};

pub struct CodeGenerator {
    assembly: String,
}

impl CodeGenerator {
    pub fn new() -> Self {
        CodeGenerator {
            assembly: String::new(),
        }
    }

    pub fn generate(&mut self, expr: &Expression) -> String {
        self.assembly.clear();
        self.assembly.push_str("global _start\n");
        self.assembly.push_str("section .text\n");
        self.assembly.push_str("_start:\n");
    
        self.generate_expression(expr);
    
        // Exit syscall
        self.assembly.push_str("    mov rax, 60\n");
        self.assembly.push_str("    mov rdi, 0\n");
        self.assembly.push_str("    syscall\n");
    
        self.assembly.clone()
    }

    fn generate_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::Number(n) => {
                self.assembly.push_str(&format!("    mov rax, {}\n", n));
            }
            Expression::Float(f) => {
                // Note: This is a placeholder. Floating-point code generation is complex.
                self.assembly.push_str(&format!("    ; Float value: {}\n", f));
            }
            Expression::String(s) => {
                // Note: String handling requires more complex code generation
                self.assembly.push_str(&format!("    ; String value: {}\n", s));
            }
            Expression::Boolean(b) => {
                self.assembly.push_str(&format!("    mov rax, {}\n", if *b { 1 } else { 0 }));
            }
            Expression::List(_) => {
                // Note: List handling requires runtime support
                self.assembly.push_str("    ; List generation not implemented\n");
            }
            Expression::Map(_) => {
                // Note: Map handling requires runtime support
                self.assembly.push_str("    ; Map generation not implemented\n");
            }
            Expression::Identifier(id) => {
                // Note: Identifier resolution requires symbol table
                self.assembly.push_str(&format!("    ; Identifier: {}\n", id));
            }
            Expression::FunctionCall { function: _, arguments: _ } => {
                // Note: Function call generation requires more complex runtime support
                self.assembly.push_str("    ; Function call not implemented\n");
            }
            Expression::FunctionDefinition { name: _, parameters: _, body: _ } => {
                // Note: Function definition requires more complex code generation
                self.assembly.push_str("    ; Function definition not implemented\n");
            }
            Expression::BinaryOp { left, operator, right } => {
                self.generate_expression(right);
                self.assembly.push_str("    push rax\n");
                self.generate_expression(left);
                self.assembly.push_str("    pop rbx\n");
            
                match operator {
                    Operator::Add => self.assembly.push_str("    add rax, rbx\n"),
                    Operator::Subtract => self.assembly.push_str("    sub rax, rbx\n"),
                    Operator::Multiply => self.assembly.push_str("    imul rax, rbx\n"),
                    Operator::Divide => {
                        self.assembly.push_str("    xor rdx, rdx\n");
                        self.assembly.push_str("    idiv rbx\n");
                    }
                    Operator::Power => {
                        self.assembly.push_str("    mov rcx, rbx\n");
                        self.assembly.push_str("    mov rbx, rax\n");
                        self.assembly.push_str("    mov rax, 1\n");
                        self.assembly.push_str("power_loop:\n");
                        self.assembly.push_str("    imul rax, rbx\n");
                        self.assembly.push_str("    loop power_loop\n");
                    }
                    Operator::Equals => self.assembly.push_str("    ; Equals comparison not implemented\n"),
                    Operator::NotEquals => self.assembly.push_str("    ; Not equals comparison not implemented\n"),
                    Operator::LessThan => self.assembly.push_str("    ; Less than comparison not implemented\n"),
                    Operator::GreaterThan => self.assembly.push_str("    ; Greater than comparison not implemented\n"),
                }
            }
            Expression::LogCall { level, message } => {
                // Placeholder for log function code generation
                self.generate_expression(message);
                match level {
                    LogLevel::Debug => self.assembly.push_str("    ; LogDebug not implemented\n"),
                    LogLevel::Info => self.assembly.push_str("    ; LogInfo not implemented\n"),
                    LogLevel::Warn => self.assembly.push_str("    ; LogWarn not implemented\n"),
                    LogLevel::Error => self.assembly.push_str("    ; LogError not implemented\n"),
                }
            }
        }
    }
}
