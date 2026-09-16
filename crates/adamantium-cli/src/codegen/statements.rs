use super::*;

impl Generator {
    pub(super) fn instructions(&mut self, instructions: &[Instruction], function: &Function) {
        for instruction in instructions {
            let mark = self.next_slot;
            match instruction {
                Instruction::Noop => (),
                Instruction::Assign(slot, value) => {
                    self.expression(value);
                    self.clone_class(value.ty);
                    self.store(*slot);
                }
                Instruction::Disconnect(destination, source) => {
                    self.load(*source);
                    self.store(*destination);
                }
                Instruction::Remove(object, hook) => {
                    if let Some(hook) = hook {
                        self.expression(object);
                        let receiver = self.save();
                        self.call_saved(hook, &[receiver]);
                    }
                }
                Instruction::Clamp(slot, low, high) => {
                    self.expression(low);
                    let low = self.save();
                    self.expression(high);
                    let high = self.save();
                    self.evaluate(
                        6,
                        function.types[*slot],
                        function.types[*slot],
                        &[*slot, low, high],
                    );
                    self.store(*slot);
                }
                Instruction::Print(value, newline) => {
                    if let Type::Class(id) = value.ty {
                        self.print_class(value, id, *newline);
                    } else {
                        self.expression(value);
                        self.print_loaded(value.ty, *newline);
                    }
                }
                Instruction::Message(message, panic, line) => {
                    self.expression(message);
                    let message = self.save();
                    self.emit(format!(
                        "    lea rcx, {}\n    mov edx, {}\n    mov r8d, {}\n    call ad_message",
                        memory(message, 0),
                        line,
                        u8::from(*panic)
                    ));
                    if *panic {
                        let error = self.error_target().to_string();
                        self.emit(format!("    jmp {error}"));
                    }
                }
                Instruction::Call(expr) => self.expression(expr),
                Instruction::Exit(code) => {
                    if let Some(code) = code {
                        self.expression(code);
                        self.emit("    mov ecx, eax");
                    } else {
                        self.emit("    xor ecx, ecx");
                    }
                    self.emit("    call ExitProcess");
                }
                Instruction::SetField(object, index, value, hook) => {
                    self.set_class_field(object, *index, value, hook.as_deref());
                }
                Instruction::SetIndex(list, index, value, line) => {
                    self.expression(list);
                    let list = self.save();
                    self.expression(index);
                    let index = self.save();
                    self.expression(value);
                    self.clone_class(value.ty);
                    let value = self.save();
                    self.emit_list_bounds(list, index, *line);
                    self.load(list);
                    self.emit("    mov r11, rax");
                    self.load(index);
                    self.emit("    shl rax, 4\n    add r11, rax");
                    self.load(value);
                    self.emit("    mov [r11], rax\n    mov [r11 + 8], rdx");
                }
                Instruction::If(condition, yes, no) => {
                    self.if_statement(condition, yes, no, function)
                }
                Instruction::While(condition, body) => {
                    self.conditional_loop(condition, body, function, false)
                }
                Instruction::Until(condition, body) => {
                    self.conditional_loop(condition, body, function, true)
                }
                Instruction::Loop(body) => self.unconditional_loop(body, function),
                Instruction::For(slot, start, end, body) => {
                    self.range_loop(*slot, start, end, body, function)
                }
                Instruction::ForEach(slot, collection, body) => {
                    self.collection_loop(*slot, collection, body, function)
                }
                Instruction::Match(value, arms, fallback) => {
                    self.match_statement(value, arms, fallback.as_deref(), function)
                }
                Instruction::Break => {
                    self.emit(format!("    jmp {}", self.loop_stack.last().unwrap().1))
                }
                Instruction::Continue => {
                    self.emit(format!("    jmp {}", self.loop_stack.last().unwrap().0))
                }
                Instruction::Return => self.emit(format!("    jmp ad_return_{}", function.name)),
            }
            self.next_slot = mark;
        }
    }
}
