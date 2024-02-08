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

impl Tag {
    pub fn htmlify(self) -> String {
        let parsed_attrs = self
            .attrs
            .iter()
            .map(|(key, value)| {
                if let Some(value) = value.as_deref() {
                    format!("{key}=\"{value}\"")
                } else {
                    key.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(" ");

        let parsed_children = self
            .children
            .iter()
            .map(|child| child.clone().htmlify())
            .collect::<Vec<_>>()
            .join("");
        let opening = format!("{name} {parsed_attrs}", name = self.name);
        let opening_proper = opening.trim_end();
        let final_str = format!(
            "<{opening_proper}>{content}{parsed_children}</{name}>",
            name = self.name,
            content = self.content.as_deref().unwrap_or(""),
        );

        final_str
    }
}
