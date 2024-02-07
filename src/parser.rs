use crate::lexer::{Lexer, Token};
use crate::{ast::*, errors::*};
use aott::input::SpannedInput;
use aott::{prelude::*, select};
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
fn tag(input: Tokens) -> (Tag, bool) {
    let mut name = "fuckyou".to_string();
    let mut has_dedented = false;

    if let Token::Text(name_) = input.peek()? {
        input.skip()?;
        name = name_;
    }
    println!("starting on {}", name);

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

    let final_tag = Tag {
        children: {
            if let Ok(Token::Newline) = input.peek() {
                let offset = input.save();
                input.skip()?;

                match input.peek() {
                    Ok(Token::Indent) => {
                        println!("{name} indenting");
                        input.skip()?;
                        file(input)?
                    }
                    Ok(Token::Dedent) => {
                        println!("{name} dedenting");
                        has_dedented = true;
                        input.skip()?;
                        vec![]
                    }
                    _ => {
                        input.rewind(offset);
                        println!("{name} newline but no children");
                        vec![]
                    }
                }
            } else {
                println!("{name} no newline and no children");
                vec![]
            }
        },
        name,
        attrs,
        classes,
        content,
        id,
    };

    println!("returning: {:?}", final_tag);

    Ok((final_tag, has_dedented))
}

#[parser(extras=Extra)]
fn file(input: Tokens) -> Vec<Tag> {
    let mut top_level_tags: Vec<Tag> = vec![];

    loop {
        if input.peek().is_ok() {
            let (returned_tag, has_dedented) = tag(input)?;
            let name_cp = returned_tag.name.clone();
            top_level_tags.push(returned_tag);

            if has_dedented {
                println!("Dedent detected on {}, breaking", name_cp);
                break;
            }
            if let Ok(Token::Newline) = input.peek() {
                input.skip()?;
            } else {
                println!("peek ok but no newline");
                break;
            }
        } else {
            println!("peek no longer ok");
            break;
        }
    }
    Ok(top_level_tags)
}

pub fn parse(input: &str) -> Result<Vec<Tag>, crate::errors::Error> {
    //let lexer = crate::lexer::Lexer::new(input);
    //println!("{:#?}\n\n", lexer.collect::<Vec<_>>());

    file.parse_with_context(
        Stream::from_iter(crate::lexer::Lexer::new(input)).spanned(input.len()..input.len()),
        input.to_owned(),
    )
    .map_err(Into::into)
}
