use std::{
    borrow::Borrow,
    hash::{Hash, Hasher},
    ops::Deref,
};

use serde_json::Value;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Topic(String);

impl Topic {
    pub(crate) fn is_match(&self, other: &Topic) -> bool {
        let mut topic_parts = self.split(':');
        let mut other_parts = other.split(':');

        loop {
            match (topic_parts.next(), other_parts.next()) {
                (Some("*"), _) => return true,
                (Some(self_part), Some(topic_part)) if self_part == topic_part => continue,
                (None, None) => return true,
                _ => return false,
            }
        }
    }
}

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

impl Deref for Topic {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
