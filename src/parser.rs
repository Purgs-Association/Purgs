use crate::lexer::{Lexer, Token};
use crate::{ast::*, errors::*};
use aott::input::SpannedInput;
use aott::{prelude::*, select};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::ops::Range;
use std::vec;

#[parser(extras=Extra)]
fn parse_string(input: Tokens) -> String {
    let quote_char = select!(Token::Quote(slice) => slice).parse_with(input)?;

    let start = input.offset();

    loop {
        if let Token::Quote(slice) = input.peek()? {
            if slice == quote_char {
                let result = input.context()[input.span_since(start)].to_owned();
                input.skip()?;
                return Ok(result);
            }
        } else {
            input.skip()?;
        }
    }
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
    let before = input.offset();

    loop {
        if let Token::Newline = input.peek()? {
            return Ok(input.context()[input.span_since(before)].to_owned());
        } else {
            input.skip()?;
        }
    }
}

struct Extra;
impl<I: InputType<Token = Token, Span = Range<usize>>> ParserExtras<I> for Extra {
    type Context = String;
    type Error = ParserError;
}

type Tokens = SpannedInput<Token, Range<usize>, Stream<Lexer>>;

#[parser(extras=Extra)]
fn tag(input: Tokens) -> Tag {
    let mut name = "fuckyou".to_string();
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
        children: {
            let mut children = vec![];
            if let Ok((Token::Newline, Token::Indent)) = input.peek().and_then(|a| {
                input.skip()?;
                Ok((a, input.peek()?))
            }) {
                input.skip()?;
                loop {
                    children.push(tag(input)?);
                    if let Ok(Token::Dedent) = input.peek() {
                        input.skip()?;
                        break;
                    }
                }
            }
            children
        },
        classes,
        content,
        id,
    })
}

#[parser(extras=Extra)]
fn file(input: Tokens) -> Vec<Tag> {
    let mut top_level_tags: Vec<Tag> = vec![];

    loop {
        if input.peek().is_ok() {
            top_level_tags.push(tag(input)?);
            if let Ok(Token::Newline) = input.peek() {
                input.skip()?;
            } else {
                break;
            }
        } else {
            break;
        }
    }
    Ok(top_level_tags)

    /*
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
            Token::Indent => {
                input.skip()?;
                current_indent += 1;
            }

            _ => {
                let dif = current_indent - prev_indent;

                let parsed_tag = tag(input)?;

                match dif.cmp(&0) {
                    Ordering::Greater => {
                        if let Some(parent) = parent_stack.last_mut() {
                            println!("pushing {} to {}", parsed_tag.name, parent.name);
                            parent.children.push(parsed_tag.clone());
                        }
                        parent_stack.push(parsed_tag);
                    }

                    Ordering::Equal => {
                        parent_stack.pop();
                        if let Some(parent) = parent_stack.last_mut() {
                            println!("pushing {} to {}", parsed_tag.name, parent.name);
                            parent.children.push(parsed_tag.clone());
                        } else {
                            top_level_tags.push(parsed_tag.clone());
                        }
                        parent_stack.push(parsed_tag);
                    }

                    Ordering::Less => {
                        for _ in (dif)..1 {
                            parent_stack.pop();
                        }
                        if let Some(parent) = parent_stack.last_mut() {
                            println!("pushing {} to {}", parsed_tag.name, parent.name);
                            parent.children.push(parsed_tag.clone());
                        } else {
                            top_level_tags.push(parsed_tag.clone());
                        }
                        parent_stack.push(parsed_tag);
                    }
                }

                prev_indent = current_indent;
                current_indent = 0;
            }
        }
    }
    */
}

pub fn parse(input: &str) -> Result<Vec<Tag>, crate::errors::Error> {
    file.parse_with_context(
        Stream::from_iter(crate::lexer::Lexer::new(input)).spanned(input.len()..input.len()),
        input.to_owned(),
    )
    .map_err(Into::into)
}
