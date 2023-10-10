use aott::derive::IntoString;
use logos::Logos;

#[derive(Clone, Logos, Debug, PartialEq, IntoString)]
pub enum Token {
    #[regex(r"[ ]{4}|\t", |lex| lex.slice().to_owned())]
    Indent(String),

    #[regex(r"\n")]
    Newline,

    #[regex(r"[a-zA-Z0-9]+", |lex| lex.slice().to_owned())]
    Text(String),

    #[regex(r#""|'"#, |lex| lex.slice().to_owned())]
    Quote(String),

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

    Error(String),
}
