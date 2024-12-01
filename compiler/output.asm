global _start
section .data
section .text
_start:
    ; Print string: Hello, World!
    mov rax, 1       ; syscall number for write
    mov rdi, 1       ; file descriptor (stdout)
    mov rsi, string_0
    mov rdx, 14
    syscall
    mov rax, 60
    mov rdi, 0
    syscall
