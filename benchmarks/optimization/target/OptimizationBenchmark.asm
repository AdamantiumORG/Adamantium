; Adamantium Windows x64 entry point and typed runtime calls.
bits 64
default rel
global main
extern SetConsoleOutputCP
extern ExitProcess
extern ad_evaluate
extern ad_print
extern ad_message
extern ad_object_new
extern ad_object_clone
extern ad_list_error
extern ad_optional_error
extern ad_parse_arguments
extern ad_try_begin
extern ad_try_end
extern ad_has_error
extern ad_is_trying
extern ad_package_call

section .text
main:
    sub rsp, 56
    mov [rsp + 40], rcx
    mov [rsp + 48], rdx
    mov ecx, 65001
    call SetConsoleOutputCP
    mov rcx, [rsp + 40]
    mov rdx, [rsp + 48]
    call ad_cli_main
    xor ecx, ecx
    call ExitProcess

; Runtime helpers return a nonzero exit code on failure.
ad_exit_error:
    mov ecx, eax
    call ExitProcess
ad_cli_main:
    push rbp
    mov rbp, rsp
    sub rsp, 64
    lea r8, [rel ad_argument_specs]
    mov r9d, 0
    lea rax, [rbp - 16]
    mov [rsp + 32], rax
    call ad_parse_arguments
    test eax, eax
    jnz ad_exit_error
    call ad_fun_main
    mov rsp, rbp
    pop rbp
    ret

ad_fun_main:
    push rbp
    mov rbp, rsp
    mov r11, 176
ad_probe_main:
    cmp r11, 4096
    jb ad_tail_main
    sub rsp, 4096
    test byte [rsp], 0
    sub r11, 4096
    jmp ad_probe_main
ad_tail_main:
    sub rsp, r11
    test byte [rsp], 0
    mov rax, 0
    mov rdx, 0
    mov [rbp - 32], rax
    mov [rbp - 24], rdx
    mov rax, 10000
    mov rdx, 0
    mov [rbp - 48], rax
    mov [rbp - 40], rdx
ad_for_condition_0:
    mov qword [rbp - 128], 0
    mov qword [rbp - 120], 0
    mov qword [rbp - 112], 0
    mov qword [rbp - 104], 0
    mov qword [rbp - 96], 0
    mov qword [rbp - 88], 0
    mov qword [rbp - 80], 0
    mov qword [rbp - 72], 0
    mov qword [rbp - 64], 0
    mov qword [rbp - 56], 0
    mov rax, [rbp - 32]
    mov rdx, [rbp - 24]
    mov [rbp - 128], rax
    mov [rbp - 120], rdx
    mov rax, [rbp - 48]
    mov rdx, [rbp - 40]
    mov [rbp - 112], rax
    mov [rbp - 104], rdx
    mov dword [rbp - 64], 9
    mov dword [rbp - 60], 16
    mov dword [rbp - 56], 16
    mov dword [rbp - 52], 0
    lea rcx, [rbp - 128]
    call ad_evaluate
    test eax, eax
    jnz ad_error_main
    mov rax, [rbp - 80]
    mov rdx, [rbp - 72]
    test rax, rax
    jz ad_loop_end_2
    mov rax, 0
    mov rdx, 0
    mov [rbp - 16], rax
    mov [rbp - 8], rdx
ad_for_increment_1:
    mov rax, 1
    xor edx, edx
    mov [rbp - 64], rax
    mov [rbp - 56], rdx
    mov qword [rbp - 144], 0
    mov qword [rbp - 136], 0
    mov qword [rbp - 128], 0
    mov qword [rbp - 120], 0
    mov qword [rbp - 112], 0
    mov qword [rbp - 104], 0
    mov qword [rbp - 96], 0
    mov qword [rbp - 88], 0
    mov qword [rbp - 80], 0
    mov qword [rbp - 72], 0
    mov rax, [rbp - 32]
    mov rdx, [rbp - 24]
    mov [rbp - 144], rax
    mov [rbp - 136], rdx
    mov rax, [rbp - 64]
    mov rdx, [rbp - 56]
    mov [rbp - 128], rax
    mov [rbp - 120], rdx
    mov dword [rbp - 80], 0
    mov dword [rbp - 76], 16
    mov dword [rbp - 72], 16
    mov dword [rbp - 68], 0
    lea rcx, [rbp - 144]
    call ad_evaluate
    test eax, eax
    jnz ad_error_main
    mov rax, [rbp - 96]
    mov rdx, [rbp - 88]
    mov [rbp - 32], rax
    mov [rbp - 24], rdx
    jmp ad_for_condition_0
ad_loop_end_2:
    jmp ad_return_main
ad_error_main:
    call ad_is_trying
    test eax, eax
    jnz ad_return_main
    mov ecx, 2
    call ExitProcess
ad_return_main:
    xor eax, eax
    xor edx, edx
    mov rsp, rbp
    pop rbp
    ret
section .rdata
ad_argument_specs:
    dq 0
