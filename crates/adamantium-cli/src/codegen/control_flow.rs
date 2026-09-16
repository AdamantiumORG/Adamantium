use super::*;

impl Generator {
    pub(super) fn if_statement(
        &mut self,
        condition: &Expression,
        yes: &[Instruction],
        no: &[Instruction],
        function: &Function,
    ) {
        let else_label = self.label("else");
        let end = self.label("if_end");
        self.expression(condition);
        self.emit(format!("    test rax, rax\n    jz {else_label}"));
        self.instructions(yes, function);
        self.emit(format!("    jmp {end}\n{else_label}:"));
        self.instructions(no, function);
        self.emit(format!("{end}:"));
    }

    pub(super) fn conditional_loop(
        &mut self,
        condition: &Expression,
        body: &[Instruction],
        function: &Function,
        until: bool,
    ) {
        let start = self.label("condition");
        let end = self.label("loop_end");
        self.loop_stack.push((start.clone(), end.clone()));
        self.emit(format!("{start}:"));
        self.expression(condition);
        let jump = if until { "jnz" } else { "jz" };
        self.emit(format!("    test rax, rax\n    {jump} {end}"));
        self.instructions(body, function);
        self.emit(format!("    jmp {start}\n{end}:"));
        self.loop_stack.pop();
    }

    pub(super) fn unconditional_loop(&mut self, body: &[Instruction], function: &Function) {
        let start = self.label("loop");
        let end = self.label("loop_end");
        self.loop_stack.push((start.clone(), end.clone()));
        self.emit(format!("{start}:"));
        self.instructions(body, function);
        self.emit(format!("    jmp {start}\n{end}:"));
        self.loop_stack.pop();
    }

    pub(super) fn range_loop(
        &mut self,
        slot: usize,
        start_value: &Expression,
        end_value: &Expression,
        body: &[Instruction],
        function: &Function,
    ) {
        self.expression(start_value);
        self.store(slot);
        self.expression(end_value);
        let end_slot = self.save();
        let condition = self.label("for_condition");
        let increment = self.label("for_increment");
        let end = self.label("loop_end");
        self.loop_stack.push((increment.clone(), end.clone()));
        self.emit(format!("{condition}:"));
        self.evaluate(9, start_value.ty, start_value.ty, &[slot, end_slot]);
        self.emit(format!("    test rax, rax\n    jz {end}"));
        self.instructions(body, function);
        self.emit(format!("{increment}:\n    mov rax, 1\n    xor edx, edx"));
        let one = self.save();
        self.evaluate(0, start_value.ty, start_value.ty, &[slot, one]);
        self.store(slot);
        self.emit(format!("    jmp {condition}\n{end}:"));
        self.loop_stack.pop();
    }

    pub(super) fn collection_loop(
        &mut self,
        slot: usize,
        collection: &Expression,
        body: &[Instruction],
        function: &Function,
    ) {
        let Type::List(inner) = collection.ty else {
            unreachable!("typed for-each collection must be a List")
        };
        let element_ty = Type::from_id(inner).expect("checked List element type");
        self.expression(collection);
        let list = self.save();
        self.emit("    xor eax, eax\n    xor edx, edx");
        let index = self.save();
        let condition = self.label("for_each_condition");
        let increment = self.label("for_each_increment");
        let end = self.label("loop_end");
        self.loop_stack.push((increment.clone(), end.clone()));
        self.emit(format!("{condition}:"));
        self.load(index);
        self.emit(format!("    cmp rax, {}\n    jae {end}", memory(list, 8)));
        self.load(list);
        self.emit("    mov r11, rax");
        self.load(index);
        self.emit("    shl rax, 4\n    add r11, rax\n    mov rax, [r11]\n    mov rdx, [r11 + 8]");
        self.clone_class(element_ty);
        self.store(slot);
        self.instructions(body, function);
        self.emit(format!(
            "{increment}:\n    mov rax, {}\n    inc rax\n    xor edx, edx",
            memory(index, 0)
        ));
        self.store(index);
        self.emit(format!("    jmp {condition}\n{end}:"));
        self.loop_stack.pop();
    }

    pub(super) fn match_statement(
        &mut self,
        value: &Expression,
        arms: &[adamantium_ir::typed::MatchBranch<Type, crate::types::Value>],
        fallback: Option<&[Instruction]>,
        function: &Function,
    ) {
        self.expression(value);
        let matched_value = self.save();
        let end = self.label("match_end");
        for (pattern, body) in arms {
            let next = self.label("match_next");
            self.expression(pattern);
            let pattern = self.save();
            self.evaluate(7, value.ty, value.ty, &[matched_value, pattern]);
            self.emit(format!("    test rax, rax\n    jz {next}"));
            self.instructions(body, function);
            self.emit(format!("    jmp {end}\n{next}:"));
        }
        if let Some(body) = fallback {
            self.instructions(body, function);
        }
        self.emit(format!("{end}:"));
    }
}
