use clap::Parser;
use dotenv::dotenv;
use handler::Cli;
use serde::{Deserialize, Serialize};
use db::create_pool;

mod handler;
mod document;
mod embedding;
mod chat;
mod db;
mod vector_store;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let config = read_config()?;
    let args = Cli::parse();
    let pool = create_pool().await?;
    handler::handler(args, config, pool).await?;
    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    // Database
    db_url: String,

    // Query
    n_results: usize,

    // Chunk
    chunk_size: u32,
    
    // Embedding
    embedding_dim: u32,
    batch: u32,

    zhipu_url: String,
    zhipu_embedding_model: String,
    zhipu_api_key: String,

    // chat
    deepseek_url: String,
    deepseek_chat_model: String,
    deepseek_api_key: String,

    // Prompt
    system_prompt: String,
}

fn read_config() -> anyhow::Result<Config> {
    Ok(config::Config::builder()
        .add_source(config::File::with_name("config"))
        .build()?
        .try_deserialize::<Config>()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_prompt() -> anyhow::Result<()> {
        let config = read_config()?;
        println!("{}", config.system_prompt);
        Ok(())
    }

    #[test]
    fn test_read_config() -> anyhow::Result<()> {
        let config = read_config()?;
        println!("{:?}", config);
        Ok(())
    }
}