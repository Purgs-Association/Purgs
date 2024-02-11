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

const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

impl Tag {
    pub fn htmlify(&self) -> String {
        let Self {
            name,
            attrs,
            id,
            classes,
            children,
            content,
        } = self;
        let classes = classes.join(" ");

        let parsed_attrs = attrs
            .iter()
            // remap these so we can append id and classes by chaining and not a new hashmap
            .map(|(key, value)| (key.as_str(), value.as_deref()))
            .chain(id.as_deref().map(|id| ("id", Some(id))))
            .chain(if classes.is_empty() {
                None
            } else {
                Some(("class", Some(classes.as_str())))
            })
            .map(|(key, value)| {
                if let Some(value) = value {
                    format!(" {key}=\"{value}\"")
                } else {
                    key.to_string()
                }
            })
            .collect::<Vec<_>>()
            // first one starts with space, which separates it from {name}, others spaced normally
            .join("");

        if VOID_ELEMENTS.contains(&name.as_str()) {
            debug_assert!(children.is_empty(), "Void elements must not have children");
            debug_assert!(content.is_none(), "Void elements must not have content");

            return format!("<{name}{parsed_attrs}/>");
        }

        let parsed_children = children
            .iter()
            .map(|child| child.htmlify())
            .collect::<Vec<_>>()
            .join("");

        format!(
            "<{name}{parsed_attrs}>{content}{parsed_children}</{name}>",
            content = content.as_deref().unwrap_or(""),
        )
    }
}
