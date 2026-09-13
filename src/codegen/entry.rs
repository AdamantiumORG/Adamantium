use super::*;

fn cli_entry(function: &Function) -> String {
    let count = function.parameters;
    let output_size = count.max(1) * 16;
    let frame = (output_size + 63) / 16 * 16;
    let mut text = format!(
        "ad_cli_main:\n    push rbp\n    mov rbp, rsp\n    sub rsp, {frame}\n    lea r8, [rel ad_argument_specs]\n    mov r9d, {count}\n    lea rax, [rbp - {output_size}]\n    mov [rsp + 32], rax\n    call ad_parse_arguments\n    test eax, eax\n    jnz ad_exit_error\n"
    );
    if count != 0 {
        text.push_str(&format!("    sub rsp, {}\n", count * 16));
        for index in 0..count {
            text.push_str(&format!(
                "    mov rax, [rbp - {}]\n    mov rdx, [rbp - {}]\n    mov [rsp + {}], rax\n    mov [rsp + {}], rdx\n",
                output_size - index * 16,
                output_size - index * 16 - 8,
                index * 16,
                index * 16 + 8
            ));
        }
    }
    text.push_str(&format!("    call ad_fun_{}\n", function.name));
    if count != 0 {
        text.push_str(&format!("    add rsp, {}\n", count * 16));
    }
    text.push_str("    mov rsp, rbp\n    pop rbp\n    ret\n");
    text
}

pub fn assembly_entry(program: &Program, entry: &str) -> String {
    let runtime = if cfg!(target_os = "linux") {
        include_str!("../runtime-linux.asm")
    } else {
        include_str!("../runtime.asm")
    };
    let runtime_length = runtime.len();
    let mut generator = Generator {
        text: runtime.into(),
        data: Vec::new(),
        next_slot: 0,
        max_slot: 0,
        class_sizes: program.class_sizes.clone(),
        classes: program.classes.clone(),
        next_label: 0,
        loop_stack: Vec::new(),
        error_targets: Vec::new(),
        current_function_name: String::new(),
        current_function_types: Vec::new(),
        package_functions: program.package_functions.clone(),
    };
    let main = program
        .functions
        .iter()
        .find(|function| function.name == entry)
        .expect("checked entry function must exist");
    generator.emit(cli_entry(main));
    for function in &program.functions {
        generator.function(function);
    }
    generator.emit("section .rdata");
    for (i, bytes) in generator.data.iter().enumerate() {
        generator.text.push_str(&format!("ad_string_{i}:\n"));
        for chunk in bytes.chunks(32) {
            generator.text.push_str(&format!(
                "    db {}\n",
                chunk
                    .iter()
                    .map(u8::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        generator.text.push_str("    db 0\n");
    }
    generator.text.push_str("ad_argument_specs:\n");
    if main.parameters == 0 {
        generator.text.push_str("    dq 0\n");
    }
    for (index, (name, ty)) in main
        .parameter_names
        .iter()
        .zip(&main.types[..main.parameters])
        .enumerate()
    {
        generator.text.push_str(&format!(
            "    dq ad_argument_name_{index}\n    dq {}\n    dd {}\n    dd {}\n",
            name.len(),
            ty.id(),
            u8::from(matches!(ty, Type::Optional(_)))
        ));
    }
    for (index, name) in main.parameter_names.iter().enumerate() {
        let bytes = name
            .bytes()
            .map(|byte| byte.to_string())
            .collect::<Vec<_>>();
        generator.text.push_str(&format!(
            "ad_argument_name_{index}:\n    db {}\n",
            bytes.join(", ")
        ));
    }
    if cfg!(target_os = "linux") {
        let mut generated = generator.text.split_off(runtime_length);
        for (windows_name, linux_name) in [
            ("ExitProcess", "ad_linux_exit"),
            ("ad_evaluate", "ad_linux_evaluate"),
            ("ad_print", "ad_linux_print"),
            ("ad_message", "ad_linux_message"),
            ("ad_object_new", "ad_linux_object_new"),
            ("ad_object_clone", "ad_linux_object_clone"),
            ("ad_list_error", "ad_linux_list_error"),
            ("ad_optional_error", "ad_linux_optional_error"),
            ("ad_parse_arguments", "ad_linux_parse_arguments"),
            ("ad_try_begin", "ad_linux_try_begin"),
            ("ad_try_end", "ad_linux_try_end"),
            ("ad_has_error", "ad_linux_has_error"),
            ("ad_is_trying", "ad_linux_is_trying"),
            ("ad_package_call", "ad_linux_package_call"),
        ] {
            generated = generated.replace(
                &format!("call {windows_name}"),
                &format!("call {linux_name}"),
            );
        }
        generator.text.push_str(&generated);
    }
    generator.text
}
