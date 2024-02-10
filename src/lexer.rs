use std::ops::Range;
use tracing::*;

use aott::derive::IntoString;
use logos::Logos;

use std::cmp::Ordering;

use crate::iter::NanoPeek;

#[derive(Clone, Logos, Debug, PartialEq, IntoString)]
pub enum SmallToken {
    #[regex(r"\t+|([ ]{4})+")]
    Indent,

    #[regex(r"\n")]
    Newline,

    #[regex(r"[a-zA-Z0-9-]+")]
    Text,

    #[regex(r#""|'"#)]
    Quote,

    #[token(".")]
    Dot,

    #[token("#")]
    Hash,

    #[token("(")]
    OpenParen,

    #[token(")")]
    CloseParen,

    #[token("=")]
    Equals,

    #[token(",")]
    Comma,

    #[token(" ")]
    WhiteSpace,
}

#[derive(Clone, Logos, Debug, PartialEq)]
pub enum Token {
    Indent,
    Dedent,
    Newline,
    Text(String),
    Quote(String),
    Dot,
    Hash,
    OpenParen,
    CloseParen,
    Equals,
    Comma,
    WhiteSpace,
    Error(String),
}

/*
#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub kind: Kind,
    pub slice: String,
}*/

pub struct Lexer {
    // HACK: We need 'static here to get rid of lifetimes in the parsers
    logos: NanoPeek<logos::Lexer<'static, SmallToken>>,
    indent: usize,
    just_dedented: bool,
    dedents_left: usize,
}

impl Iterator for Lexer {
    type Item = (Token, Range<usize>);

    #[instrument(skip(self), level = "trace", ret, fields(self.dedents_left, self.indent, self.just_dedented, slice = self.logos.inner.slice()))]
    fn next(&mut self) -> Option<Self::Item> {
        if self.dedents_left > 0 {
            self.dedents_left -= 1;
            if self.dedents_left == 0 {
                self.just_dedented = true;
            }
            return Some((Token::Dedent, self.logos.inner.span()));
        }
        // just_dedented makes Newline->Dedent into Newline->Dedent->Newline so the parser doesn't suffer so put it after other checks that emit a dedent
        if self.just_dedented {
            self.just_dedented = false;
            return Some((Token::Newline, self.logos.inner.span()));
        }
        let Ok(token) = self.logos.next()? else {
            return Some((
                Token::Error(self.logos.inner.slice().to_owned()),
                self.logos.inner.span(),
            ));
        };

        let kind = match token {
            SmallToken::Dot => Token::Dot,
            SmallToken::Hash => Token::Hash,
            SmallToken::OpenParen => Token::OpenParen,
            SmallToken::CloseParen => Token::CloseParen,
            SmallToken::Equals => Token::Equals,
            SmallToken::Comma => Token::Comma,
            SmallToken::WhiteSpace => Token::WhiteSpace,
            SmallToken::Quote => Token::Quote(self.logos.inner.slice().to_owned()),
            SmallToken::Text => Token::Text(self.logos.inner.slice().to_owned()),
            SmallToken::Newline => {
                if let Some(Ok(SmallToken::Indent)) = self.logos.peek() {
                } else if self.indent > 0 {
                    self.dedents_left = self.indent;
                }
                Token::Newline
            }
            SmallToken::Indent => {
                let slice = self.logos.inner.slice();
                let indent_len = if slice.starts_with('\t') {
                    slice.len()
                } else {
                    slice.len() / 4
                };

                let token = match indent_len.cmp(&self.indent) {
                    Ordering::Greater => Token::Indent,
                    Ordering::Equal => return self.next(),
                    Ordering::Less => {
                        self.dedents_left = (self.indent - indent_len).saturating_sub(1);
                        self.just_dedented = true;
                        Token::Dedent
                    }
                };

                self.indent = indent_len;
                token
            }
        };

        Some((kind, self.logos.inner.span()))
    }
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            // FIXME: We don't deallocate this but currently I don't care
            logos: NanoPeek::new(SmallToken::lexer(Box::leak::<'static>(
                input.to_owned().into_boxed_str(),
            ))),
            indent: 0,
            just_dedented: false,
            dedents_left: 0,
        }
    }
}
