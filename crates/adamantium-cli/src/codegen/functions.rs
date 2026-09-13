use super::*;

impl Generator {
    pub(super) fn function(&mut self, function: &Function) {
        self.current_function_name.clone_from(&function.name);
        self.current_function_types.clone_from(&function.types);
        self.next_slot = function.types.len();
        self.max_slot = self.next_slot;
        let start = self.text.len();
        let error = format!("ad_error_{}", function.name);
        self.error_targets.push(error.clone());
        for slot in 0..function.parameters {
            self.emit(format!(
                "    mov rax, [rbp + {}]\n    mov rdx, [rbp + {}]",
                16 + slot * 16,
                24 + slot * 16
            ));
            self.store(slot);
        }
        self.instructions(&function.instructions, function);
        self.error_targets.pop();
        self.emit(format!(
            "    jmp ad_return_{}\n{error}:\n    call ad_is_trying\n    test eax, eax\n    jnz ad_return_{}\n    mov ecx, 2\n    call ExitProcess",
            function.name,
            function.name
        ));
        self.emit(format!("ad_return_{}:", function.name));
        if let Some(slot) = function.result {
            self.load(slot);
        } else {
            self.emit("    xor eax, eax\n    xor edx, edx");
        }
        self.emit("    mov rsp, rbp\n    pop rbp\n    ret");
        let frame = self.max_slot * 16 + 32;
        self.text.insert_str(start,&format!("ad_fun_{}:\n    push rbp\n    mov rbp, rsp\n    mov r11, {frame}\nad_probe_{}:\n    cmp r11, 4096\n    jb ad_tail_{}\n    sub rsp, 4096\n    test byte [rsp], 0\n    sub r11, 4096\n    jmp ad_probe_{}\nad_tail_{}:\n    sub rsp, r11\n    test byte [rsp], 0\n",function.name,function.name,function.name,function.name,function.name));
    }
    pub(super) fn call_saved(&mut self, name: &str, slots: &[usize]) {
        let size = slots.len() * 16;
        if size != 0 {
            self.emit(format!("    sub rsp, {size}"));
        }
        for (index, slot) in slots.iter().enumerate() {
            self.load(*slot);
            self.emit(format!(
                "    mov [rsp + {}], rax\n    mov [rsp + {}], rdx",
                index * 16,
                index * 16 + 8
            ));
        }
        self.emit(format!("    call ad_fun_{name}"));
        if size != 0 {
            self.emit(format!("    add rsp, {size}"));
        }
        let returned = self.save();
        let error = self.error_target().to_string();
        self.emit(format!(
            "    call ad_has_error\n    test eax, eax\n    jnz {error}"
        ));
        self.load(returned);
    }
}
