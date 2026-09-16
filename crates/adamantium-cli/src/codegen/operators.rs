use super::*;

impl Generator {
    pub(super) fn unary_operation(&mut self, expr: &Expression, value: &Expression) {
        let mark = self.next_slot;
        self.expression(value);
        let slot = self.save();
        let operation = if matches!(expr.kind, Kind::Negate(_)) {
            4
        } else {
            5
        };
        self.evaluate(operation, expr.ty, value.ty, &[slot]);
        self.next_slot = mark;
    }

    pub(super) fn logical_not(&mut self, value: &Expression) {
        self.expression(value);
        self.emit("    test rax, rax\n    sete al\n    movzx eax, al\n    xor edx, edx");
    }

    pub(super) fn logical_operation(
        &mut self,
        operator: LogicalOperator,
        left: &Expression,
        right: &Expression,
    ) {
        let skip = self.label("logical_skip");
        let end = self.label("logical_end");
        self.expression(left);
        self.emit("    test rax, rax");
        match operator {
            LogicalOperator::And => self.emit(format!("    jz {skip}")),
            LogicalOperator::Or => self.emit(format!("    jnz {skip}")),
        }
        self.expression(right);
        self.emit(format!(
            "    jmp {end}\n{skip}:\n    mov eax, {}\n    xor edx, edx\n{end}:",
            u8::from(matches!(operator, LogicalOperator::Or))
        ));
    }

    pub(super) fn binary_operation(
        &mut self,
        operator: Operator,
        result_type: Type,
        left: &Expression,
        right: &Expression,
    ) {
        let mark = self.next_slot;
        self.expression(left);
        let left = self.save();
        self.expression(right);
        let right = self.save();
        let operation = match operator {
            Operator::Add => 0,
            Operator::Subtract => 1,
            Operator::Multiply => 2,
            Operator::Divide => 3,
            Operator::Remainder => 13,
        };
        self.evaluate(operation, result_type, result_type, &[left, right]);
        self.next_slot = mark;
    }

    pub(super) fn comparison(
        &mut self,
        comparison: Comparison,
        left: &Expression,
        right: &Expression,
    ) {
        let mark = self.next_slot;
        self.expression(left);
        let left_slot = self.save();
        self.expression(right);
        let right_slot = self.save();
        let operation = match comparison {
            Comparison::Equal => 7,
            Comparison::NotEqual => 8,
            Comparison::Less => 9,
            Comparison::LessEqual => 10,
            Comparison::Greater => 11,
            Comparison::GreaterEqual => 12,
        };
        self.evaluate(operation, left.ty, left.ty, &[left_slot, right_slot]);
        self.next_slot = mark;
    }
}
