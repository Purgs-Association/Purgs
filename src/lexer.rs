use std::ops::Range;

use aott::derive::IntoString;
use itertools::Itertools;
use logos::Logos;

use crate::iter::NanoPeek;

#[derive(Clone, Logos, Debug, PartialEq, IntoString)]
pub enum SmallToken {
    #[regex(r"\t|[ ]{4}")]
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
    logos: NanoPeek<logos::Lexer<'static, SmallToken>>,
    indent: usize,
    next_no_indent: bool,
    input: &'static str,
}

impl Iterator for Lexer {
    type Item = (Token, Range<usize>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_no_indent {
            self.next_no_indent = false;
            return Some((Token::Dedent, self.logos.inner.span()));
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
                } else {
                    self.next_no_indent = true;
                }
                Token::Newline
            }
            SmallToken::Indent => {
                let indent = self
                    .logos
                    .peeking_take_while(|token| matches!(token, Ok(SmallToken::Indent)))
                    .count()
                    + 1;
                let token = if indent > self.indent {
                    Token::Indent
                } else {
                    Token::Dedent
                };

                self.indent = indent;
                token
            }
        };

        println!(
            "kind: {:?}\nslice: {:?}\n\n",
            kind,
            self.logos.inner.slice()
        );

        Some((kind, self.logos.inner.span()))
    }
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let input_owned = &*Box::leak::<'static>(input.to_owned().into_boxed_str());

        Self {
            logos: NanoPeek::new(SmallToken::lexer(input_owned)),
            indent: 0,
            next_no_indent: false,
            input: input_owned,
        }
    }
}
