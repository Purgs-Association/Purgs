use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    pub name: String,
    pub attrs: HashMap<String, Option<String>>,
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub children: Vec<Tag>,
    pub content: Option<String>,
}

fn htmlify(tag: &Tag) -> String {
    let parsed_attrs = tag
        .attrs
        .iter()
        .map(|(key, value)| {
            if let Some(value) = value.as_deref() {
                format!("{key}=\"{value}\"")
            } else {
                format!("{key}")
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    let parsed_children = tag
        .children
        .iter()
        .map(htmlify)
        .collect::<Vec<_>>()
        .join("");

    let final_str = format!(
        "<{name} {parsed_attrs}>{content}{parsed_children}</{name}>",
        name = tag.name,
        content = tag.content.as_deref().unwrap_or(""),
    );

    final_str
}
