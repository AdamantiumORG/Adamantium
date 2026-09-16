mod assembly;
mod entry;
mod expressions;
mod functions;
mod lists;
mod statements;

pub use entry::assembly_entry;

use adamantium_ir::typed::{
    ArithmeticOperator as Operator, ComparisonOperator as Comparison, LogicalOperator,
};

use crate::{
    typed::{ClassInfo, Expression, Function, Instruction, Kind, PackageFunction, Program},
    types::Type,
};

pub(super) struct Generator {
    pub(super) text: String,
    pub(super) data: Vec<Vec<u8>>,
    pub(super) next_slot: usize,
    pub(super) max_slot: usize,
    pub(super) class_sizes: Vec<usize>,
    pub(super) classes: Vec<ClassInfo>,
    pub(super) next_label: usize,
    pub(super) loop_stack: Vec<(String, String)>,
    pub(super) error_targets: Vec<String>,
    pub(super) current_function_name: String,
    pub(super) current_function_types: Vec<Type>,
    pub(super) package_functions: std::collections::HashMap<String, PackageFunction>,
}
pub(super) fn memory(slot: usize, offset: usize) -> String {
    format!("[rbp - {}]", (slot + 1) * 16 - offset)
}
