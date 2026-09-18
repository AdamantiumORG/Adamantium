use super::*;

pub(super) fn optimize(text: String) -> String {
    let mut result = String::with_capacity(text.len());
    for line in text.lines() {
        let instruction = line.trim();
        if matches!(
            instruction,
            "mov rax, rax" | "mov rdx, rdx" | "add rsp, 0" | "sub rsp, 0"
        ) {
            continue;
        }
        result.push_str(line);
        result.push('\n');
    }
    result
}

impl Generator {
    pub(super) fn error_target(&self) -> &str {
        self.error_targets
            .last()
            .expect("code generation requires an error target")
    }
    pub(super) fn label(&mut self, prefix: &str) -> String {
        let label = format!("ad_{prefix}_{}", self.next_label);
        self.next_label += 1;
        label
    }
    pub(super) fn emit(&mut self, text: impl AsRef<str>) {
        self.text.push_str(text.as_ref());
        self.text.push('\n');
    }
    pub(super) fn reserve(&mut self, count: usize) -> usize {
        self.next_slot += count;
        self.max_slot = self.max_slot.max(self.next_slot);
        self.next_slot - 1
    }
    pub(super) fn store(&mut self, slot: usize) {
        self.emit(format!(
            "    mov {}, rax\n    mov {}, rdx",
            memory(slot, 0),
            memory(slot, 8)
        ));
    }
    pub(super) fn load(&mut self, slot: usize) {
        self.emit(format!(
            "    mov rax, {}\n    mov rdx, {}",
            memory(slot, 0),
            memory(slot, 8)
        ));
    }
    pub(super) fn save(&mut self) -> usize {
        let slot = self.reserve(1);
        self.store(slot);
        slot
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn removes_redundant_register_and_stack_instructions() {
        let assembly = "start:\n    mov rax, rax\n    sub rsp, 0\n    mov rax, 1\n".to_string();
        assert_eq!(super::optimize(assembly), "start:\n    mov rax, 1\n");
    }
}
