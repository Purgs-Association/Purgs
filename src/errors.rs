use crate::lexer::Token;
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
    #[error("filter faoled at {}..{}: {location}, with Token {token:?}", .span.start, .span.end)]
    FilterFailed {
        span: Span,
        location: &'static std::panic::Location<'static>,
        token: Token,
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

impl<I: InputType<Token = Token>> aott::error::Error<I> for ParserError {
    fn filter_failed(
        span: Range<usize>,
        location: &'static std::panic::Location<'static>,
        token: Token,
    ) -> Self {
        Self::FilterFailed {
            span,
            location,
            token,
        }
    }
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
