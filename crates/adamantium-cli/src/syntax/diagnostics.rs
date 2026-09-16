#[derive(Clone, Copy, Debug)]
pub struct Position {
    pub(super) line: usize,
    pub(super) column: usize,
}
impl Position {
    pub fn error(self, message: impl std::fmt::Display) -> String {
        format!("{}:{}: {message}", self.line, self.column)
    }
    pub fn line(self) -> usize {
        self.line
    }
}
