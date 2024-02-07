use std::ops::Range;

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
    logos: NanoPeek<logos::Lexer<'static, SmallToken>>,
    indent: usize,
    just_dedented: bool,
    next_no_indent: bool, // For the last dedent there is no indent token to check, so we need to know if we need to emit a dedent
    dedents_left: usize,
}

impl Iterator for Lexer {
    type Item = (Token, Range<usize>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.just_dedented {
            self.just_dedented = false;
            println!("Newline from just_dedented\n");
            return Some((Token::Newline, self.logos.inner.span()));
        }
        if self.next_no_indent {
            println!(
                "dedent from next_no_indent\nDedents left: {}\n",
                self.dedents_left
            );
            self.next_no_indent = false;
            self.just_dedented = true;
            return Some((Token::Dedent, self.logos.inner.span()));
        }
        if self.dedents_left > 0 {
            println!(
                "dedent from dedents_left\nDedents left: {}\n",
                self.dedents_left
            );
            self.dedents_left -= 1;
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
                } else if self.indent > 0 {
                    self.next_no_indent = true;
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

        println!("{:?} {:?}\n", kind, self.logos.inner.slice());

        Some((kind, self.logos.inner.span()))
    }
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let input_owned = &*Box::leak::<'static>(input.to_owned().into_boxed_str());

        Self {
            logos: NanoPeek::new(SmallToken::lexer(input_owned)),
            indent: 0,
            just_dedented: false,
            next_no_indent: false,
            dedents_left: 0,
        }
    }
}
