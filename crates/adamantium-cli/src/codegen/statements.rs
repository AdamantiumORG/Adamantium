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
                    self.expression(object);
                    let receiver = self.save();
                    self.expression(value);
                    self.clone_class(value.ty);
                    let field_value = self.save();
                    self.load(receiver);
                    self.emit("    mov r11, rax");
                    self.load(field_value);
                    self.emit(format!(
                        "    mov [r11 + {}], rax\n    mov [r11 + {}], rdx",
                        index * 16,
                        index * 16 + 8
                    ));
                    if let Some(hook) = hook {
                        self.call_saved(hook, &[receiver]);
                    }
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
                    let else_label = self.label("else");
                    let end = self.label("if_end");
                    self.expression(condition);
                    self.emit(format!("    test rax, rax\n    jz {else_label}"));
                    self.instructions(yes, function);
                    self.emit(format!("    jmp {end}\n{else_label}:"));
                    self.instructions(no, function);
                    self.emit(format!("{end}:"));
                }
                Instruction::While(condition, body) | Instruction::Until(condition, body) => {
                    let start = self.label("condition");
                    let end = self.label("loop_end");
                    self.loop_stack.push((start.clone(), end.clone()));
                    self.emit(format!("{start}:"));
                    self.expression(condition);
                    let jump = if matches!(instruction, Instruction::While(_, _)) {
                        "jz"
                    } else {
                        "jnz"
                    };
                    self.emit(format!("    test rax, rax\n    {jump} {end}"));
                    self.instructions(body, function);
                    self.emit(format!("    jmp {start}\n{end}:"));
                    self.loop_stack.pop();
                }
                Instruction::Loop(body) => {
                    let start = self.label("loop");
                    let end = self.label("loop_end");
                    self.loop_stack.push((start.clone(), end.clone()));
                    self.emit(format!("{start}:"));
                    self.instructions(body, function);
                    self.emit(format!("    jmp {start}\n{end}:"));
                    self.loop_stack.pop();
                }
                Instruction::For(slot, start_value, end_value, body) => {
                    self.expression(start_value);
                    self.store(*slot);
                    self.expression(end_value);
                    let end_slot = self.save();
                    let condition = self.label("for_condition");
                    let increment = self.label("for_increment");
                    let end = self.label("loop_end");
                    self.loop_stack.push((increment.clone(), end.clone()));
                    self.emit(format!("{condition}:"));
                    self.evaluate(9, start_value.ty, start_value.ty, &[*slot, end_slot]);
                    self.emit(format!("    test rax, rax\n    jz {end}"));
                    self.instructions(body, function);
                    self.emit(format!("{increment}:\n    mov rax, 1\n    xor edx, edx"));
                    let one = self.save();
                    self.evaluate(0, start_value.ty, start_value.ty, &[*slot, one]);
                    self.store(*slot);
                    self.emit(format!("    jmp {condition}\n{end}:"));
                    self.loop_stack.pop();
                }
                Instruction::ForEach(slot, collection, body) => {
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
                    self.emit(
                        "    shl rax, 4\n    add r11, rax\n    mov rax, [r11]\n    mov rdx, [r11 + 8]",
                    );
                    self.clone_class(element_ty);
                    self.store(*slot);
                    self.instructions(body, function);
                    self.emit(format!(
                        "{increment}:\n    mov rax, {}\n    inc rax\n    xor edx, edx",
                        memory(index, 0)
                    ));
                    self.store(index);
                    self.emit(format!("    jmp {condition}\n{end}:"));
                    self.loop_stack.pop();
                }
                Instruction::Match(value, arms, fallback) => {
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
