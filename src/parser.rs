use crate::lexer::Token;
use crate::{ast::*, errors::*};
use aott::{prelude::*, select};
use logos::Logos;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::vec;

#[parser(extras=Extra)]
fn parse_string(input: Tokens) -> String {
    let quote_char = select!(Token::Quote(text) => text).parse_with(input)?;

    let mut string = String::new();

    loop {
        match input.peek()? {
            Token::Quote(qu) => {
                if qu == quote_char {
                    input.skip()?;
                    break;
                }
            }
            token => {
                input.skip()?;
                string.push_str(&Into::<String>::into(token))
            }
        }
    }

    Ok(string)
}

#[parser(extras=Extra)]
fn parse_attributes(input: Tokens) -> HashMap<String, Option<String>> {
    let mut attrs = HashMap::new();

    loop {
        if let Token::CloseParen = input.peek()? {
            input.skip()?;
            return Ok(attrs);
        }
        let attr_name = select!(Token::Text(text) => text).parse_with(input)?;
        if let Token::Equals = input.peek()? {
            input.skip()?;

            let attr_value = parse_string(input)?;
            attrs.insert(attr_name, Some(attr_value));
        } else {
            attrs.insert(attr_name, None);
        }

        if let Token::Comma = input.peek()? {
            input.skip()?;
        }

        if let Token::WhiteSpace = input.peek()? {
            input.skip()?;
        }
    }
}

#[parser(extras=Extra)]
fn parse_classes(input: Tokens) -> Vec<String> {
    let mut classes = vec![];

    while let Token::Dot = input.peek()? {
        input.skip()?;
        let class = select!(Token::Text(text) => text).parse_with(input)?;
        classes.push(class);
    }

    Ok(classes)
}

#[parser(extras=Extra)]
fn parse_content(input: Tokens) -> String {
    let mut content = String::new();

    loop {
        match input.peek()? {
            Token::Newline => break,
            token => {
                input.skip()?;
                content.push_str(&Into::<String>::into(token))
            }
        }
    }

    Ok(content)
}

struct Extra;
impl<I: InputType<Token = Token>> ParserExtras<I> for Extra {
    type Context = u32;
    type Error = ParserError;
}

type Tokens = Stream<std::vec::IntoIter<Token>>;

#[parser(extras=Extra)]
fn tag(input: Tokens) -> Tag {
    let mut name = "div".to_string();
    if let Token::Text(name_) = input.peek()? {
        input.skip()?;
        name = name_;
    }

    let mut id = None;

    if let Ok(Token::Hash) = input.peek() {
        input.skip()?;
        id = Some(select!(Token::Text(text) => text).parse_with(input)?);
    }

    let mut classes = vec![];

    if let Ok(Token::Dot) = input.peek() {
        classes = parse_classes(input)?;
    }

    let mut attrs = HashMap::new();

    if let Ok(Token::OpenParen) = input.peek() {
        input.skip()?;
        attrs = parse_attributes(input)?;
    }

    let mut content: Option<String> = None;
    if let Ok(Token::WhiteSpace) = input.peek() {
        input.skip()?;
        content = Some(parse_content(input)?);
    }

    Ok(Tag {
        name,
        attrs,
        children: vec![],
        classes,
        content,
        id,
    })
}

#[parser(extras=Extra)]
fn file(input: Tokens) -> Tag {
    let mut og_parent = Tag {
        name: "html".to_string(),
        attrs: HashMap::new(),
        children: vec![],
        classes: vec![],
        content: None,
        id: None,
    };

    let mut parent_stack: Vec<Tag> = vec![];
    let mut current_indent = 0;
    let mut prev_indent = 0;

    loop {
        let Ok(peeked) = input.peek() else { break };
        match peeked {
            Token::Newline => {
                input.skip()?;
                continue;
            }
            Token::Indent(_indent) => {
                input.skip()?;
                current_indent += 1;
            }

            _ => {
                let dif = current_indent - prev_indent;

                match dif.cmp(&0) {
                    Ordering::Greater => {}
                    Ordering::Equal => {
                        if let Some(parent) = parent_stack[..].last_mut() {
                            parent.children.push(tag(input)?);
                        } else {
                            og_parent.children.push(tag(input)?);
                        }
                    }
                    Ordering::Less => {}
                }

                og_parent.children.push(tag(input)?);

                prev_indent = current_indent;
                current_indent = 0;
            }
        }
    }

    Ok(og_parent)
}

pub fn parse(input: &str) -> Result<Tag, crate::errors::Error> {
    let mut tokens = vec![];
    let mut lexer = Token::lexer(input);

    while let Some(token) = lexer.next() {
        tokens.push(token.unwrap_or_else(|()| Token::Error(lexer.slice().to_owned())));
    }

    Ok(file.parse_with_context(Stream::from_iter(tokens), 0)?)
}
