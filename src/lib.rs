use regex::Regex;
use std::cmp::Ordering;
pub mod ast;
pub mod errors;
pub mod iter;
mod lexer;
mod parser;

pub use parser::parse;

const SELF_CLOSING_TAGS: [&str; 16] = [
    "area", "base", "br", "col", "embed", "hr", "img", "input", "keygen", "link", "menuitem",
    "meta", "param", "source", "track", "wbr",
];

pub fn purgs(content: &str) -> String {
    let mut final_str = "".to_owned();
    let mut stack: Vec<String> = vec![];
    let stuff: Vec<_> = content.split("\n").collect();

    let re = Regex::new(r#"([a-z\d]+)?(?:#([a-zA-Z-_\d]+))?(?:\.([a-zA-Z-_\d]+))?(\(?(?:(?:([a-zA-Z-_]+)=(?:"|')([^"]*)(?:"|'))(?:,\s*|\s+)?)*\))?(?:\s(.+))?"#).unwrap();
    let re_attr = Regex::new(r#"([a-zA-Z-_]+)=(?:"|')([^"]*)(?:"|')"#).unwrap();

    let mut prev_indent = -1;

    for item in stuff.iter() {
        let filtered = item.replace("	", "");
        if !filtered.is_empty() {
            let old_len = item.len() as i32;
            let indent: i32 = old_len - filtered.len() as i32;
            let dif = indent - prev_indent;

            let mut tag = "div".to_string();
            let mut content = "".to_string();
            let mut attrs = "".to_string();
            let mut classes = vec![];

            let mut id = "".to_string();

            for capture in re.captures_iter(&filtered) {
                if let Some(result) = capture.get(1) {
                    tag = result.as_str().to_string();
                }
                if let Some(result) = capture.get(2) {
                    id = result.as_str().to_string();
                }
                if let Some(result) = capture.get(3) {
                    classes.push(result.as_str().to_string());
                }
                if let Some(result) = capture.get(4) {
                    let attrs_str = result.as_str().to_string();
                    for attr in re_attr.captures_iter(&attrs_str) {
                        attrs += &format!(
                            "{}=\"{}\" ",
                            attr.get(1).unwrap().as_str(),
                            attr.get(2).unwrap().as_str()
                        )
                        .to_string();
                    }
                }

                if let Some(group6) = capture.get(7) {
                    content = group6.as_str().to_string();
                }
            }

            if !id.is_empty() {
                attrs += &format!("id=\"{}\" ", id).to_string();
            }
            if !classes.is_empty() {
                attrs += &format!("class=\"{}\" ", classes.join(" ")).to_string();
            }

            let mut inside = format!("{} {}", tag, attrs);
            let inside_len = inside.len();
            inside = String::from(&inside[0..inside_len - 1]);

            if SELF_CLOSING_TAGS.contains(&tag.as_str()) {
                inside += "/";
            }

            let final_tag = &format!("<{}>{}", inside, content);

            match dif.cmp(&0) {
                Ordering::Greater => {
                    final_str += final_tag;
                }
                Ordering::Equal => {
                    let closing = stack.pop().unwrap();
                    if !closing.is_empty() {
                        final_str += &format!("</{}>", closing);
                    }
                    final_str += final_tag;
                }
                Ordering::Less => {
                    for _ in (dif)..1 {
                        let closing = stack.pop().unwrap();
                        if !closing.is_empty() {
                            final_str += &format!("</{}>", closing);
                        }
                    }
                    final_str += final_tag;
                }
            }

            if !SELF_CLOSING_TAGS.contains(&tag.as_str()) {
                stack.push(tag);
            } else {
                stack.push("".to_string());
            }
            prev_indent = indent;
        }
    }

    for _ in 0..stack.len() {
        let closing = stack.pop().unwrap();
        if !closing.is_empty() {
            final_str += &format!("</{}>", closing);
        }
    }

    final_str
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::ast::Tag;

    fn check_str(input: &str) {
        println!("Testing input \"{input}\"");
        println!("Old: {}", crate::purgs(input));
        println!(
            "New: {:?}",
            crate::parse(input).unwrap_or_else(|error| {
                match &error {
                    crate::errors::Error::Parser(
                        crate::errors::ParserError::Filtering { span, .. }
                        | crate::errors::ParserError::ExpectedEOF { span, .. }
                        | crate::errors::ParserError::ExpectedToken { span, .. }
                        | crate::errors::ParserError::UnexpectedEOF { span, .. },
                    ) => eprintln!(
                        "at {span_start}..{span_end}, slice: \"{slice}\" (the span + 3 chars before it)",
                        span_start = span.start,
                        span_end = span.end,
                        slice = &input[(span.start.saturating_sub(3))..(span.end)]
                    ),
                    _ => unreachable!(),
                };

                panic!("Error: {error}")
            })
        );
    }

    fn test_str(input: &str, expected: Vec<Tag>) {
        println!("Testing input \"{input}\"");
        let out = crate::parse(input).unwrap_or_else(|error| {
                match &error {
                    crate::errors::Error::Parser(
                        crate::errors::ParserError::Filtering { span, .. }
                        | crate::errors::ParserError::ExpectedEOF { span, .. }
                        | crate::errors::ParserError::ExpectedToken { span, .. }
                        | crate::errors::ParserError::UnexpectedEOF { span, .. },
                    ) => eprintln!(
                        "at {span_start}..{span_end}, slice: \"{slice}\" (the span + 3 chars before it)",
                        span_start = span.start,
                        span_end = span.end,
                        slice = &input[(span.start.saturating_sub(3))..(span.end)]
                    ),
                    _ => unreachable!(),
                };

                panic!("Error: {error}")
            });
        println!("Actual: {:?}", out);

        assert_eq!(out, expected);
    }

    #[test]
    fn simple() {
        check_str("html\n\thead\n\t\tmeta(width=\"device-width=true\")\n\tbody\n\t\tdiv#content.hello Hello World");
    }

    #[test]
    fn multi_dedent() {
        test_str("html\n\thead\n\t\tmeta(width=\"device-width=true\")\n\tbody\n\t\tdiv#content.hello Hello World\nanotertoplevelthinglolhaha", vec![Tag { name: "html".to_string(), attrs: HashMap::new(), id: None, classes: vec![], children: vec![Tag { name: "head".to_string(), attrs: HashMap::new(), id: None, classes: vec![], children: vec![Tag { name: "meta".to_string(), attrs: HashMap::from_iter([("width".to_string(), Some("device-width=true".to_string()))]), id: None, classes: vec![], children: vec![], content: None }], content: None }], content: None }, Tag { name: "anotertoplevelthinglolhaha".to_string(), attrs: HashMap::new(), id: None, classes: vec![], children: vec![], content: None }]);
    }
}
