use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod deepseek;
pub mod apis;

pub use deepseek::FormatType;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    User,
    Assistant,
    System,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Talk {
    pub role: Role,
    pub content: String,
}

// Define chat schema

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LLMOutput {
    pub content: String,
    pub call: bool,
    pub api: Option<String>,
    pub params: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LLMInput {
    pub content: String,
    pub api_output: bool,
}

impl Talk {
    pub fn new(role: Role, content: String) -> Self {
        Self { role, content }
    }
}

impl LLMInput {
    pub fn new(content: String, api_output: bool) -> Self {
        Self { content, api_output }
    }
}

#[cfg(test)]
mod tests {
    use crate::chat::deepseek::ChatClient;
    use crate::read_config;

    use super::*;

    #[tokio::test]
    async fn test_completion() -> anyhow::Result<()> {
        let config = read_config()?;
        let client = ChatClient::from_config(&config);
        let mut messages: Vec<Talk> = vec! [Talk::new(Role::User, "hello".to_string())];
        client.get_completion(&mut messages, FormatType::Text).await?;
        println!("{:?}", messages.last());
        Ok(())
    }
}
