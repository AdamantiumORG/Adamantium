use super::*;

impl Generator {
    pub(super) fn expression(&mut self, expr: &Expression) {
        match &expr.kind {
            Kind::Constant(value) => self.emit(format!(
                "    mov rax, {}\n    mov rdx, {}",
                value.lo, value.hi
            )),
            Kind::String(bytes) => {
                let index = self.data.len();
                self.data.push(bytes.clone());
                self.emit(format!(
                    "    lea rax, [rel ad_string_{index}]\n    mov rdx, {}",
                    bytes.len()
                ));
            }
            Kind::Variable(slot) => self.load(*slot),
            Kind::Call(name, arguments) => self.call(name, arguments),
            Kind::Try(instructions) => {
                let result = self.reserve(1);
                let failed = self.label("try_failed");
                let done = self.label("try_done");
                self.emit("    call ad_try_begin");
                self.error_targets.push(failed.clone());
                let context = Function {
                    name: self.current_function_name.clone(),
                    parameters: 0,
                    result: None,
                    types: self.current_function_types.clone(),
                    instructions: Vec::new(),
                    parameter_names: Vec::new(),
                };
                self.instructions(instructions, &context);
                self.error_targets.pop();
                self.emit(format!(
                    "    lea rcx, {}\n    call ad_try_end\n    jmp {done}\n{failed}:\n    lea rcx, {}\n    call ad_try_end\n{done}:\n    mov rax, {}\n    mov rdx, {}",
                    memory(result, 0),
                    memory(result, 0),
                    memory(result, 0),
                    memory(result, 8)
                ));
            }
            Kind::Address(slot) => {
                self.emit(format!(
                    "    lea rax, {}\n    xor edx, edx",
                    memory(*slot, 0)
                ));
            }
            Kind::Dereference(value) => {
                self.expression(value);
                self.emit("    mov r11, rax\n    mov rax, [r11]\n    mov rdx, [r11 + 8]");
            }
            Kind::Construct(id, fields, constructor) => {
                let mark = self.next_slot;
                let mut values = Vec::new();
                for field in fields {
                    self.expression(field);
                    self.clone_class(field.ty);
                    values.push(self.save());
                }
                self.emit(format!(
                    "    mov ecx, {}\n    call ad_object_new\n    xor edx, edx",
                    fields.len()
                ));
                let object = self.save();
                for (index, value) in values.iter().enumerate() {
                    self.load(object);
                    self.emit("    mov r11, rax");
                    self.load(*value);
                    self.emit(format!(
                        "    mov [r11 + {}], rax\n    mov [r11 + {}], rdx",
                        index * 16,
                        index * 16 + 8
                    ));
                }
                self.load(object);
                let receiver = self.save();
                self.call_saved(constructor, &[receiver]);
                self.load(object);
                self.next_slot = mark;
                let _ = id;
            }
            Kind::List(values) => {
                let mark = self.next_slot;
                let mut elements = Vec::new();
                for value in values {
                    self.expression(value);
                    self.clone_class(value.ty);
                    elements.push(self.save());
                }
                self.emit(format!(
                    "    mov ecx, {}\n    call ad_object_new",
                    values.len()
                ));
                let list = self.save();
                for (index, element) in elements.iter().enumerate() {
                    self.load(list);
                    self.emit("    mov r11, rax");
                    self.load(*element);
                    self.emit(format!(
                        "    mov [r11 + {}], rax\n    mov [r11 + {}], rdx",
                        index * 16,
                        index * 16 + 8
                    ));
                }
                self.load(list);
                self.emit(format!("    mov rdx, {}", values.len()));
                self.next_slot = mark;
            }
            Kind::Index(list, index, line) => {
                let mark = self.next_slot;
                self.expression(list);
                let list = self.save();
                self.expression(index);
                let index = self.save();
                self.emit_list_bounds(list, index, *line);
                self.load(list);
                self.emit("    mov r11, rax");
                self.load(index);
                self.emit(
                    "    shl rax, 4\n    add r11, rax\n    mov rax, [r11]\n    mov rdx, [r11 + 8]",
                );
                self.next_slot = mark;
            }
            Kind::Field(object, index) => {
                self.expression(object);
                self.emit(format!(
                    "    mov r11, rax\n    mov rax, [r11 + {}]\n    mov rdx, [r11 + {}]",
                    index * 16,
                    index * 16 + 8
                ));
            }
            Kind::MethodCall(name, object, arguments) => {
                let mark = self.next_slot;
                self.expression(object);
                let mut slots = vec![self.save()];
                for argument in arguments {
                    self.expression(argument);
                    self.clone_class(argument.ty);
                    slots.push(self.save());
                }
                self.call_saved(name, &slots);
                self.next_slot = mark;
            }
            Kind::Negate(value) | Kind::Convert(value) => {
                let mark = self.next_slot;
                self.expression(value);
                let slot = self.save();
                let op = if matches!(expr.kind, Kind::Negate(_)) {
                    4
                } else {
                    5
                };
                self.evaluate(op, expr.ty, value.ty, &[slot]);
                self.next_slot = mark;
            }
            Kind::Unwrap(value, line) => {
                let compact = self.label("optional_compact");
                let done = self.label("optional_unwrapped");
                self.expression(value);
                let error = self.error_target().to_string();
                self.emit(format!("    test rdx, rdx\n    jnz {compact}\n    mov ecx, {line}\n    call ad_optional_error\n    jmp {error}\n{compact}:\n    cmp rdx, 2\n    jne {done}\n    mov r11, rax\n    mov rax, [r11]\n    mov rdx, [r11 + 8]\n{done}:"));
            }
            Kind::Not(value) => {
                self.expression(value);
                self.emit("    test rax, rax\n    sete al\n    movzx eax, al\n    xor edx, edx");
            }
            Kind::Logical(operator, a, b) => {
                let skip = self.label("logical_skip");
                let end = self.label("logical_end");
                self.expression(a);
                self.emit("    test rax, rax");
                match operator {
                    LogicalOperator::And => self.emit(format!("    jz {skip}")),
                    LogicalOperator::Or => self.emit(format!("    jnz {skip}")),
                }
                self.expression(b);
                self.emit(format!(
                    "    jmp {end}\n{skip}:\n    mov eax, {}\n    xor edx, edx\n{end}:",
                    u8::from(matches!(operator, LogicalOperator::Or))
                ));
            }
            Kind::Binary(op, a, b) => {
                let mark = self.next_slot;
                self.expression(a);
                let left = self.save();
                self.expression(b);
                let right = self.save();
                let op = match op {
                    Operator::Add => 0,
                    Operator::Subtract => 1,
                    Operator::Multiply => 2,
                    Operator::Divide => 3,
                    Operator::Remainder => 13,
                };
                self.evaluate(op, expr.ty, expr.ty, &[left, right]);
                self.next_slot = mark;
            }
            Kind::Compare(comparison, a, b) => {
                let mark = self.next_slot;
                self.expression(a);
                let left = self.save();
                self.expression(b);
                let right = self.save();
                let operation = match comparison {
                    Comparison::Equal => 7,
                    Comparison::NotEqual => 8,
                    Comparison::Less => 9,
                    Comparison::LessEqual => 10,
                    Comparison::Greater => 11,
                    Comparison::GreaterEqual => 12,
                };
                self.evaluate(operation, a.ty, a.ty, &[left, right]);
                self.next_slot = mark;
            }
        }
    }
}
