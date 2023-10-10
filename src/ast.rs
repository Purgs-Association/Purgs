use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct File(pub Vec<Tag>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    pub name: String,
    pub attrs: HashMap<String, Option<String>>,
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub children: Vec<Tag>,
    pub content: Option<String>,
}
