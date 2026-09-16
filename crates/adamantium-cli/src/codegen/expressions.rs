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
                self.construct_class(*id, fields, constructor)
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
            Kind::StringLength(value, line) => {
                let mark = self.next_slot;
                self.expression(value);
                let value = self.save();
                self.evaluate_at(14, Type::U64, Type::String, &[value], *line);
                self.next_slot = mark;
            }
            Kind::Index(list, index, line) => {
                let mark = self.next_slot;
                let string = list.ty == Type::String;
                self.expression(list);
                let list = self.save();
                self.expression(index);
                let index = self.save();
                if string {
                    self.evaluate_at(15, Type::String, Type::String, &[list, index], *line);
                } else {
                    self.emit_list_bounds(list, index, *line);
                    self.load(list);
                    self.emit("    mov r11, rax");
                    self.load(index);
                    self.emit(
                        "    shl rax, 4\n    add r11, rax\n    mov rax, [r11]\n    mov rdx, [r11 + 8]",
                    );
                }
                self.next_slot = mark;
            }
            Kind::Field(object, index) => self.class_field(object, *index),
            Kind::MethodCall(name, object, arguments) => self.class_method(name, object, arguments),
            Kind::Negate(value) | Kind::Convert(value) => self.unary_operation(expr, value),
            Kind::Unwrap(value, line) => {
                let compact = self.label("optional_compact");
                let done = self.label("optional_unwrapped");
                self.expression(value);
                let error = self.error_target().to_string();
                self.emit(format!("    test rdx, rdx\n    jnz {compact}\n    mov ecx, {line}\n    call ad_optional_error\n    jmp {error}\n{compact}:\n    cmp rdx, 2\n    jne {done}\n    mov r11, rax\n    mov rax, [r11]\n    mov rdx, [r11 + 8]\n{done}:"));
            }
            Kind::Not(value) => self.logical_not(value),
            Kind::Logical(operator, a, b) => self.logical_operation(*operator, a, b),
            Kind::Binary(operator, a, b) => self.binary_operation(*operator, expr.ty, a, b),
            Kind::Compare(comparison, a, b) => self.comparison(*comparison, a, b),
        }
    }
}
