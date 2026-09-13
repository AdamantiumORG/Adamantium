use super::*;

impl Generator {
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
