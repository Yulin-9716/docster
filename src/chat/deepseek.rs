use serde::{Deserialize, Serialize};
use reqwest::Client;
use anyhow::Context;
use super::Talk;
use super::Role;
use crate::Config;

pub struct ChatClient {
    client: Client,
    api_key: String,
    model: String,
    url: String,
}

impl ChatClient {
    pub fn from_config(config: &Config) -> Self {
        Self { 
            client: Client::new(),
            api_key: config.deepseek_api_key.clone(),
            model: config.deepseek_chat_model.clone(),
            url: config.deepseek_url.clone(),
        }
    }

    pub async fn get_completion(&self, messages: &mut Vec<Talk>, format: FormatType) -> anyhow::Result<()> {
        let request = DSRequest {
            messages: messages.clone(),
            model: self.model.clone(),
            response_format: Format {
                format_type: format,
            },
        };
        let response = self.client
            .post(&self.url)
            .header("Authorization", format!("Bearer {}", &self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .with_context(|| "Failed to send request to Deepseek API")?;
        
        if response.status() != 200 { 
            return Err(anyhow::anyhow!("请求失败:\n\t错误码: {}\n\t内容: {}", response.status(), response.text().await?));
        }

        let response_body: DSResponse = response
            .json()
            .await
            .with_context(|| format!("Failed to parse Deepseek API response."))?;

        messages.push(Talk::new(Role::Assistant, response_body.choices[0].message.content.clone()));

        Ok(())
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum FormatType {
    Text,
    JsonObject,
}

#[derive(Deserialize, Serialize, Debug)]
struct Format {
    #[serde(rename = "type")]
    format_type: FormatType,
}

#[derive(Deserialize, Serialize, Debug)]
struct Choice {
    message: Talk,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DSRequest {
    messages: Vec<Talk>,
    model: String,
    response_format: Format,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DSResponse {
    choices: Vec<Choice>,
}
