use super::*;

impl Generator {
    pub(super) fn construct_class(&mut self, id: u32, fields: &[Expression], constructor: &str) {
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

    pub(super) fn class_field(&mut self, object: &Expression, index: usize) {
        self.expression(object);
        self.emit(format!(
            "    mov r11, rax\n    mov rax, [r11 + {}]\n    mov rdx, [r11 + {}]",
            index * 16,
            index * 16 + 8
        ));
    }

    pub(super) fn class_method(
        &mut self,
        name: &str,
        object: &Expression,
        arguments: &[Expression],
    ) {
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

    pub(super) fn set_class_field(
        &mut self,
        object: &Expression,
        index: usize,
        value: &Expression,
        hook: Option<&str>,
    ) {
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
}
