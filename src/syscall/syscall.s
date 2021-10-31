.global _syscall_handler
.extern syscall_handler, SYSCALL_RSP, RETURN_RSP

_syscall_handler:
    mov [RETURN_RSP], rsp
    mov rsp, [SYSCALL_RSP]
    push rcx
    push r11
    call syscall_handler
    pop r11
    pop rcx
    mov [SYSCALL_RSP], rsp
    mov rsp, [RETURN_RSP]
    sysretq