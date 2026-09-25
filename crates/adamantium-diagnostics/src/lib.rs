use adamantium_ast::Span;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Stage {
    Lexing,
    Parsing,
    NameResolution,
    TypeChecking,
    SemanticAnalysis,
    Project,
    Package,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Label {
    pub span: Span,
    pub message: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    pub code: &'static str,
    pub message: String,
    pub severity: Severity,
    pub stage: Stage,
    details: Box<DiagnosticDetails>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiagnosticDetails {
    pub primary: Label,
    pub secondary: Vec<Label>,
    pub notes: Vec<String>,
    pub help: Option<String>,
}

impl std::ops::Deref for Diagnostic {
    type Target = DiagnosticDetails;

    fn deref(&self) -> &Self::Target {
        &self.details
    }
}

impl std::ops::DerefMut for Diagnostic {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.details
    }
}

impl Diagnostic {
    pub fn error(code: &'static str, message: impl Into<String>, span: Span) -> Self {
        Self::at_stage(
            Stage::SemanticAnalysis,
            Severity::Error,
            code,
            message,
            span,
        )
    }

    pub fn at_stage(
        stage: Stage,
        severity: Severity,
        code: &'static str,
        message: impl Into<String>,
        span: Span,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            severity,
            stage,
            details: Box::new(DiagnosticDetails {
                primary: Label {
                    span,
                    message: None,
                },
                secondary: Vec::new(),
                notes: Vec::new(),
                help: None,
            }),
        }
    }

    pub fn with_primary_message(mut self, message: impl Into<String>) -> Self {
        self.primary.message = Some(message.into());
        self
    }

    pub fn with_secondary(mut self, span: Span, message: impl Into<String>) -> Self {
        self.secondary.push(Label {
            span,
            message: Some(message.into()),
        });
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn span(&self) -> Span {
        self.primary.span
    }
}

pub type NameError = Diagnostic;
pub type TypeError = Diagnostic;
pub type SemanticError = Diagnostic;
