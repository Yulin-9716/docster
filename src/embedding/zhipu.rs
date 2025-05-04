use reqwest::Client;
use serde::{Deserialize, Serialize};
use anyhow::Context;

#[derive(Debug, Clone)]
pub struct ZhipuOptions {
    api_key: String,
    url: String,
    model: String,
}

#[derive(Debug, Serialize)]
pub struct ZhipuRequest<'a> {
    input: &'a Vec<&'a str>,
    model: String,
}

#[derive(Debug, Deserialize)]
pub struct ZhipuResponse {
    data: Vec<Embed>,
}

#[derive(Debug, Deserialize)]
pub struct Embed {
    // pub index: i32,
    // pub object: String,
    pub embedding: Vec<f32>,
}

impl ZhipuOptions {
    pub fn new(api_key: &String, url: &String, model: &String) -> Self {
        ZhipuOptions { 
            api_key: api_key.clone(), 
            url: url.clone(), 
            model: model.clone(),
        }
    }
}

pub struct EmbeddingClient {
    client: Client,
    options: ZhipuOptions,
}

impl EmbeddingClient {
    pub fn new(options: ZhipuOptions) -> Self {
        Self { client: Client::new(), options}
    }

    pub async fn zhipu_embedding(&self, texts: &Vec<&str>) -> anyhow::Result<Vec<Embed>> {
        let request = ZhipuRequest {
            input: texts,
            model: self.options.model.clone(),
        };

        let response = self.client
            .post(&self.options.url)
            .header("Authorization", format!("Bearer {}", self.options.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Zhipu API")?;

        if response.status() != 200 {
            return Err(anyhow::anyhow!("Request Failed to Embedding Model: {}", response.status()))
        }

        let embedding_response = response
            .json::<ZhipuResponse>()
            .await
            .context("Failed to parse Zhipu API response")?;

        Ok(embedding_response.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;
    use crate::read_config;

    #[tokio::test]
    async fn test_api() -> anyhow::Result<()> {
        dotenv().ok();
        let config = read_config()?;
        let texts = vec!["helloworld", "hellorust"];
        let options = ZhipuOptions::new(
            &config.zhipu_api_key, &config.zhipu_url, &config.zhipu_embedding_model
        );
        let client = EmbeddingClient::new(options);
        let result = client.zhipu_embedding(&texts).await?
            .iter().map(|embed| embed.embedding.clone()).collect::<Vec<Vec<f32>>>();
        println!("{:?}", result);
        Ok(())
    }
}
