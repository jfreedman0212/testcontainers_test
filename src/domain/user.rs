use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Login {
    username: String,
    password: String,
}

impl Login {
    pub fn new(username: String, password: String) -> Self {
        Self { username, password }
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password
    }
}

#[derive(Serialize, Deserialize)]
pub struct UserInput {
    username: String,
    password: String,
    confirmed_password: String,
}

impl UserInput {
    pub fn new(username: String, password: String, confirmed_password: String) -> Self {
        Self {
            username,
            password,
            confirmed_password,
        }
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password
    }

    pub fn confirmed_password(&self) -> &str {
        &self.confirmed_password
    }
}

#[derive(Serialize, Deserialize)]
pub struct User {
    id: i64,
    username: String,
}

impl User {
    pub fn new(id: i64, username: String) -> Self {
        Self { id, username }
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn id(&self) -> i64 {
        self.id
    }
}
