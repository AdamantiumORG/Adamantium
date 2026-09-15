use super::*;

impl Generator {
    pub(super) fn clone_list(&mut self, element_ty: Type) {
        let mark = self.next_slot;
        let source = self.save();
        self.load(source);
        self.emit("    mov rcx, rax\n    call ad_object_clone");
        self.emit(format!("    mov rdx, {}", memory(source, 8)));
        let copy = self.save();

        if matches!(element_ty, Type::Class(_) | Type::List(_)) {
            self.emit("    xor eax, eax\n    xor edx, edx");
            let index = self.save();
            let condition = self.label("list_clone_condition");
            let end = self.label("list_clone_end");
            self.emit(format!("{condition}:"));
            self.load(index);
            self.emit(format!("    cmp rax, {}\n    jae {end}", memory(copy, 8)));
            self.load(copy);
            self.emit("    mov r11, rax");
            self.load(index);
            self.emit(
                "    shl rax, 4\n    add r11, rax\n    mov rax, [r11]\n    mov rdx, [r11 + 8]",
            );
            self.clone_class(element_ty);
            let element = self.save();
            self.load(copy);
            self.emit("    mov r11, rax");
            self.load(index);
            self.emit("    shl rax, 4\n    add r11, rax");
            self.load(element);
            self.emit("    mov [r11], rax\n    mov [r11 + 8], rdx");
            self.load(index);
            self.emit(format!(
                "    inc rax\n    xor edx, edx\n    mov {}, rax\n    jmp {condition}\n{end}:",
                memory(index, 0)
            ));
        }

        self.load(copy);
        self.next_slot = mark;
    }

    pub(super) fn emit_list_bounds(&mut self, list: usize, index: usize) {
        let valid = self.label("list_index_valid");
        self.load(index);
        self.emit(format!("    cmp rax, {}\n    jb {valid}", memory(list, 8)));
        let error = self.error_target().to_string();
        self.emit(format!(
            "    mov rcx, rax\n    mov rdx, {}\n    call ad_list_error\n    jmp {error}\n{valid}:",
            memory(list, 8)
        ));
    }
}
