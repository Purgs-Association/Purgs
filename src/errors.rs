use crate::lexer::Token;
use aott::error::{BuiltinLabel, Filtering};
use aott::prelude::InputType;
use std::fmt::Debug;
use std::ops::Range;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Parser error: {_0}")]
    Parser(#[from] ParserError),
    #[error("Lexer error: {_0}")]
    Lexer(#[from] LexerError),
}

pub type Span = Range<usize>;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("expected {}, found {found:?} at {}..{}", any_of(.expected), .span.start, .span.end)]
    ExpectedToken {
        expected: Vec<Token>,
        found: Token,
        span: Span,
    },
    #[error("unexpected end of file at {}..{}{}", .span.start, .span.end, .expected.as_deref().map_or_else(String::new, |expected| format!(", expected {}", any_of(expected))))]
    UnexpectedEOF {
        expected: Option<Vec<Token>>,
        span: Span,
    },
    #[error("expected end of file at {}..{}, found {found:?}", .span.start, .span.end)]
    ExpectedEOF { found: Token, span: Range<usize> },

    #[error("{label} at {}..{}, last token was {last_token:?}", .span.start, .span.end)]
    Builtin {
        span: Range<usize>,
        label: aott::error::BuiltinLabel,
        last_token: Option<Token>,
    },
    #[error("filtering {}, at {}..{}, but got {last_token:?}", .label.0, .span.start, .span.end)]
    Filtering {
        label: Filtering,
        span: Range<usize>,
        last_token: Option<Token>,
    },
}

#[derive(Error, Debug)]
#[error("lexer error at {}..{}", .0.start, .0.end)]
pub struct LexerError(pub Span);

pub fn any_of<T: Debug>(things: &[T]) -> String {
    match things {
        [el] => format!("{el:?}"),
        elements => format!("any of {elements:?}"),
    }
}

impl<I: InputType<Token = Token>> aott::error::FundamentalError<I> for ParserError {
    fn expected_eof_found(span: Range<usize>, found: Token) -> Self {
        Self::ExpectedEOF { found, span }
    }
    fn expected_token_found(span: Range<usize>, expected: Vec<Token>, found: Token) -> Self {
        Self::ExpectedToken {
            span,
            expected,
            found,
        }
    }
    fn unexpected_eof(span: Range<usize>, expected: Option<Vec<Token>>) -> Self {
        Self::UnexpectedEOF { span, expected }
    }
}

impl<I: InputType<Token = Token>> aott::error::LabelError<I, BuiltinLabel> for ParserError {
    fn from_label(span: Range<usize>, label: BuiltinLabel, last_token: Option<Token>) -> Self {
        Self::Builtin {
            span,
            label,
            last_token,
        }
    }
}

impl<I: InputType<Token = Token>> aott::error::LabelError<I, Filtering> for ParserError {
    fn from_label(span: Range<usize>, label: Filtering, last_token: Option<Token>) -> Self {
        Self::Filtering {
            span,
            label,
            last_token,
        }
    }
}
