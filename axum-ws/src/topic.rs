use std::{
    borrow::Borrow,
    hash::{Hash, Hasher},
};

use serde_json::Value;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Topic(String);

impl Hash for Topic {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl From<&str> for Topic {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for Topic {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<Topic> for Value {
    fn from(topic: Topic) -> Self {
        topic.0.into()
    }
}

impl Borrow<str> for Topic {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl Borrow<String> for Topic {
    fn borrow(&self) -> &String {
        &self.0
    }
}
