use std::{
    borrow::Borrow,
    hash::{Hash, Hasher},
};

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct UserId(String);

impl Hash for UserId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl From<&str> for UserId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for UserId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl Borrow<str> for UserId {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl Borrow<String> for UserId {
    fn borrow(&self) -> &String {
        &self.0
    }
}

#[test]
fn user_id_should_work() {
    use std::collections::HashSet;

    let mut user_ids = HashSet::new();
    user_ids.insert(UserId::from("user1"));
    user_ids.insert(UserId::from("user2"));

    let user1: &str = "user1";
    println!("Contains user1: {}", user_ids.contains(user1));

    let user2: String = "user2".to_string();
    println!("Contains user2: {}", user_ids.contains(&user2));
}
