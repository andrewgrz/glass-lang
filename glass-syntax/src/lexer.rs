use crate::codes::{INVALID_NUMBER_PARSE, UNKNOWN_LEX_ERROR, UNKNOWN_LEX_ERROR_CODE};
use crate::diagnostics::Diagnostic;
use chumsky::prelude::SimpleSpan;
use logos::{Logos, Span};

#[derive(Debug, PartialEq, Clone, Default)]
pub enum LexingError {
    NumberParseError,
    UnknownCharacter(String),
    #[default]
    Other,
}

impl LexingError {
    fn to_diagnostic(&self, span: &Span) -> Diagnostic {
        let span = SimpleSpan {
            start: span.start,
            end: span.end,
            context: (),
        };

        match self {
            LexingError::NumberParseError => Diagnostic::new_error(
                "Invalid parsing error".to_string(),
                INVALID_NUMBER_PARSE,
                span,
            ),
            LexingError::UnknownCharacter(character) => Diagnostic::new_error(
                format!("Unknown character: {}", character),
                UNKNOWN_LEX_ERROR_CODE,
                span,
            ),
            LexingError::Other => {
                Diagnostic::new_error("Lexing error".to_string(), UNKNOWN_LEX_ERROR, span)
            }
        }
    }
}

impl From<std::num::ParseIntError> for LexingError {
    fn from(_: std::num::ParseIntError) -> Self {
        LexingError::NumberParseError
    }
}

impl From<std::num::ParseFloatError> for LexingError {
    fn from(_: std::num::ParseFloatError) -> Self {
        LexingError::NumberParseError
    }
}

#[derive(Logos, Debug, PartialEq)]
#[logos(error(LexingError, callback = |lex| LexingError::UnknownCharacter(lex.slice().to_string()))
skip r"[ \t\n\f]+")] // Ignore this regex pattern between tokens
pub enum Token {
    // Keywords
    #[token("def")]
    Def,

    // Symbols
    #[token("(")]
    LeftParen,
    #[token(")")]
    RightParen,

    #[token("{")]
    LeftBrace,
    #[token("}")]
    RightBrace,

    #[token(":")]
    Colon,

    #[token(";")]
    SemiColon,

    #[token(",")]
    Comma,

    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("*")]
    Star,

    #[token("/")]
    Slash,

    // Numbers
    #[regex("-?[0-9]+", |lex| lex.slice().parse())]
    Integer(i64),

    #[regex("-?[0-9]+\\.[0-9]+", |lex| lex.slice().parse())]
    Float(f64),

    // Identifiers
    #[regex("[a-zA-Z_$][a-zA-Z0-9_*$]*", |lex| lex.slice().to_string())]
    Identifier(String),
}

#[derive(Debug, PartialEq)]
pub struct SpannedToken {
    token: Result<Token, Diagnostic>,
    span: SimpleSpan,
}

impl SpannedToken {
    pub fn new(token: Result<Token, Diagnostic>, start: usize, end: usize) -> Self {
        SpannedToken {
            token,
            span: SimpleSpan {
                start,
                end,
                context: (),
            },
        }
    }
}

/// Lex a content into a series of tokens
pub fn lex(content: &str) -> Vec<SpannedToken> {
    Token::lexer(content)
        .spanned()
        .map(|(token, span)| {
            let spanned = token.map_err(|t| t.to_diagnostic(&span));
            SpannedToken::new(spanned, span.start, span.end)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_lex_unknown() {
        let result = lex("🚀");
        assert_eq!(
            result[0],
            SpannedToken::new(
                Err(Diagnostic::new_error(
                    "Unknown character: 🚀".to_string(),
                    UNKNOWN_LEX_ERROR_CODE,
                    SimpleSpan {
                        start: 0,
                        end: 4,
                        context: ()
                    },
                )),
                0,
                4
            )
        );
    }

    #[test]
    fn text_lex_def() {
        let result = lex("def add(a, b: int){ a +b }");

        assert_eq!(result[0], SpannedToken::new(Ok(Token::Def), 0, 3));
        assert_eq!(
            result[1],
            SpannedToken::new(Ok(Token::Identifier("add".to_string())), 4, 7)
        );
        assert_eq!(result[2], SpannedToken::new(Ok(Token::LeftParen), 7, 8));
        assert_eq!(
            result[3],
            SpannedToken::new(Ok(Token::Identifier("a".to_string())), 8, 9)
        );
        assert_eq!(result[4], SpannedToken::new(Ok(Token::Comma), 9, 10));
        assert_eq!(
            result[5],
            SpannedToken::new(Ok(Token::Identifier("b".to_string())), 11, 12)
        );
        assert_eq!(result[6], SpannedToken::new(Ok(Token::Colon), 12, 13));
        assert_eq!(
            result[7],
            SpannedToken::new(Ok(Token::Identifier("int".to_string())), 14, 17)
        );
        assert_eq!(result[8], SpannedToken::new(Ok(Token::RightParen), 17, 18));
        assert_eq!(result[9], SpannedToken::new(Ok(Token::LeftBrace), 18, 19));
        assert_eq!(
            result[10],
            SpannedToken::new(Ok(Token::Identifier("a".to_string())), 20, 21)
        );
        assert_eq!(result[11], SpannedToken::new(Ok(Token::Plus), 22, 23));
        assert_eq!(
            result[12],
            SpannedToken::new(Ok(Token::Identifier("b".to_string())), 23, 24)
        );
        assert_eq!(result[13], SpannedToken::new(Ok(Token::RightBrace), 25, 26));
    }

    #[test]
    fn test_lex_number() {
        let result = lex("123");
        assert_eq!(result[0], SpannedToken::new(Ok(Token::Integer(123)), 0, 3));
    }

    #[test]
    fn test_lex_mul() {
        let result = lex("123*0.3");
        assert_eq!(result[0], SpannedToken::new(Ok(Token::Integer(123)), 0, 3));
        assert_eq!(result[1], SpannedToken::new(Ok(Token::Star), 3, 4));
        assert_eq!(result[2], SpannedToken::new(Ok(Token::Float(0.3)), 4, 7));
    }
}
