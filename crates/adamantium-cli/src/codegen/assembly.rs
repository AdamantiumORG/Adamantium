use super::*;

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
    pub(super) fn print_loaded(&mut self, ty: Type, newline: bool) {
        let slot = self.save();
        let error = self.error_target().to_string();
        self.emit(format!(
            "    lea rcx, {}\n    mov edx, {}\n    mov r8d, {}\n    call ad_print\n    test eax, eax\n    jnz {error}",
            memory(slot, 0),
            ty.id(),
            u8::from(newline)
        ));
    }
    pub(super) fn print_text(&mut self, text: &str, newline: bool) {
        let index = self.data.len();
        self.data.push(text.as_bytes().to_vec());
        self.emit(format!(
            "    lea rax, [rel ad_string_{index}]\n    mov rdx, {}",
            text.len()
        ));
        self.print_loaded(Type::String, newline);
    }
    pub(super) fn print_class(&mut self, value: &Expression, id: u32, newline: bool) {
        self.expression(value);
        let object = self.save();
        let class = self.classes[id as usize].clone();
        self.print_text(&format!("{}(", class.name), false);
        let mut first = true;
        for (index, field) in class.fields.iter().enumerate() {
            if !field.public {
                continue;
            }
            if !first {
                self.print_text(", ", false);
            }
            first = false;
            self.print_text(&format!("{}=", field.name), false);
            self.load(object);
            self.emit("    mov r11, rax");
            self.emit(format!(
                "    mov rax, [r11 + {}]\n    mov rdx, [r11 + {}]",
                index * 16,
                index * 16 + 8
            ));
            self.print_loaded(field.ty, false);
        }
        self.print_text(")", newline);
    }
    pub(super) fn clone_class(&mut self, ty: Type) {
        match ty {
            Type::Class(id) => self.emit(format!(
                "    mov rcx, rax\n    mov edx, {}\n    call ad_object_clone\n    xor edx, edx",
                self.class_sizes[id as usize]
            )),
            Type::List(inner) => self.clone_list(
                Type::from_id(inner).expect("checked List element type must be available"),
            ),
            _ => (),
        }
    }
    pub(super) fn call(&mut self, name: &str, arguments: &[Expression]) {
        if let Some(function) = self.package_functions.get(name).cloned() {
            self.package_call(&function, arguments);
            return;
        }
        let mark = self.next_slot;
        let mut slots = Vec::new();
        for argument in arguments {
            self.expression(argument);
            self.clone_class(argument.ty);
            slots.push(self.save());
        }
        let size = slots.len() * 16;
        for offset in (0..size).step_by(4096) {
            let step = (size - offset).min(4096);
            self.emit(format!("    sub rsp, {step}\n    test byte [rsp], 0"));
        }
        for (i, slot) in slots.iter().enumerate() {
            self.load(*slot);
            self.emit(format!(
                "    mov [rsp + {}], rax\n    mov [rsp + {}], rdx",
                i * 16,
                i * 16 + 8
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
        self.next_slot = mark;
    }
    pub(super) fn package_call(&mut self, function: &PackageFunction, arguments: &[Expression]) {
        let mark = self.next_slot;
        let request = self.reserve(14);
        for offset in (0..224).step_by(8) {
            self.emit(format!("    mov qword {}, 0", memory(request, offset)));
        }
        for (index, argument) in arguments.iter().enumerate() {
            self.expression(argument);
            self.emit(format!(
                "    mov {}, rax\n    mov {}, rdx\n    mov dword {}, {}",
                memory(request, 32 + index * 16),
                memory(request, 40 + index * 16),
                memory(request, 160 + index * 4),
                argument.ty.id()
            ));
        }
        for (offset, value) in [(0, &function.wasm_path), (16, &function.command)] {
            let data = self.data.len();
            self.data.push(value.as_bytes().to_vec());
            self.emit(format!(
                "    lea rax, [rel ad_string_{data}]\n    mov {}, rax\n    mov qword {}, {}",
                memory(request, offset),
                memory(request, offset + 8),
                value.len()
            ));
        }
        self.emit(format!(
            "    mov dword {}, {}\n    mov dword {}, {}\n    mov dword {}, {}",
            memory(request, 192),
            arguments.len(),
            memory(request, 196),
            function.result.id(),
            memory(request, 200),
            function.filesystem
        ));
        let error = self.error_target().to_string();
        self.emit(format!(
            "    lea rcx, {}\n    call ad_package_call\n    test eax, eax\n    jnz {error}\n    mov rax, {}\n    mov rdx, {}",
            memory(request, 0),
            memory(request, 208),
            memory(request, 216)
        ));
        self.next_slot = mark;
    }
    pub(super) fn evaluate(&mut self, operation: u32, ty: Type, from: Type, operands: &[usize]) {
        let mark = self.next_slot;
        let request = self.reserve(5);
        // Request is 80 bytes: three operands, output, operation/type/from/padding.
        for offset in (0..80).step_by(8) {
            self.emit(format!("    mov qword {}, 0", memory(request, offset)));
        }
        for (i, slot) in operands.iter().enumerate() {
            self.load(*slot);
            self.emit(format!(
                "    mov {}, rax\n    mov {}, rdx",
                memory(request, i * 16),
                memory(request, i * 16 + 8)
            ));
        }
        let error = self.error_target().to_string();
        self.emit(format!("    mov dword {}, {operation}\n    mov dword {}, {}\n    mov dword {}, {}\n    lea rcx, {}\n    call ad_evaluate\n    test eax, eax\n    jnz {error}\n    mov rax, {}\n    mov rdx, {}", memory(request,64), memory(request,68),ty.id(),memory(request,72),from.id(),memory(request,0),memory(request,48),memory(request,56)));
        self.next_slot = mark;
    }
}
