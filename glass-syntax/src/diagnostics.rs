//! Diagnostic Structs and Enums.
//!
//! Mimics the format of LSP Diagnostic where possible. Also supports easy conversions to: Ariadne error reports
//!
//! see: <https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#diagnostic>
//! see: <https://docs.rs/ariadne/latest/ariadne/>

use chumsky::prelude::SimpleSpan;

/// The level of the diagnostic that the system emitted.
///
/// For ease of translation, this enum will mirror the DiagnosticSeverity from LSP
/// see: <https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#diagnosticSeverity>
#[derive(Debug, PartialEq, Clone, Default)]
pub enum DiagnosticLevel {
    /// Reports an Error.
    /// This level means that the file will not compile. There is a breaking issue
    /// with the system. Lexing, Parsing and TypeErrors, all throw this error
    #[default]
    Error,

    /// A warning level. This will not stop the compilation from completing
    Warning,

    /// Information level. - Not Emitted at the moment
    Information,

    /// A Hint that could make the application better
    Hint,
}

/// A Diagnostic that the glass system has emitted
#[derive(Debug, PartialEq, Clone, Default)]
pub struct Diagnostic {
    /// The level of the diagnostic
    level: DiagnosticLevel,
    /// The string message for the diagnostic
    message: String,
    /// The error code of the diagnostic
    code: Option<usize>,
    /// The span of the location of the diagnostic
    span: SimpleSpan,
}

impl Diagnostic {
    pub fn new(
        level: DiagnosticLevel,
        message: String,
        code: Option<usize>,
        span: SimpleSpan,
    ) -> Self {
        Diagnostic {
            level,
            message,
            code,
            span,
        }
    }

    pub fn new_error(message: String, code: usize, span: SimpleSpan) -> Self {
        Diagnostic::new(DiagnosticLevel::Error, message, Some(code), span)
    }
}
