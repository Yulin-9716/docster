use std::collections::HashMap;
use anyhow::Context;
use chromadb::client::{ChromaAuthMethod, ChromaClient, ChromaClientOptions};
use chromadb::collection::{ChromaCollection, CollectionEntries, GetOptions, QueryOptions, QueryResult};
use serde_json::{Value, map::Map};

use crate::chat::deepseek::ChatClient;
use crate::embedding::zhipu::{EmbeddingClient, ZhipuOptions};
use crate::Config;

pub struct VectorStore {
    client: ChromaClient,
    embedding_cli: EmbeddingClient,
    chat_cli: ChatClient,

    // Config when request
    batch: u32,

    // Config when query
    n_results: usize,
}

impl VectorStore {
    fn dissimilarity(&self, v1: &[f32], v2: &[f32]) -> f32 {
        assert_eq!(v1.len(), v2.len(), "Vectors must have the same length.");

        let dot_product: f32 = v1.iter().zip(v2.iter())
            .map(|(a, b)| a * b)
            .sum();
        
        let norm_v1: f32 = v1.iter()
            .map(|x| x * x)
            .sum::<f32>()
            .sqrt();
            
        let norm_v2: f32 = v2.iter()
            .map(|x| x * x)
            .sum::<f32>()
            .sqrt();
            
        dot_product / (norm_v1 * norm_v2)
    }

    async fn get_all_ids(&self, coll_name: &str, offset: usize, limit: usize) -> anyhow::Result<Vec<String>> {
        let collection = self.client.get_collection(coll_name).await?;
        let options = GetOptions {
            offset: Some(offset),
            limit: Some(limit),
            ..Default::default()
        };
        let ids = collection.get(options).await?.ids;
        Ok(ids)
    }

    pub async fn from_config(config: &Config) -> anyhow::Result<VectorStore>
    {
        let auth = ChromaAuthMethod::None;
        let client = ChromaClient::new(
            ChromaClientOptions { url: Some(config.db_url.clone()), auth, ..Default::default() }
        ).await.with_context(|| "Database Connection Failed")?;
        let options = ZhipuOptions::new(
            &config.zhipu_api_key, &config.zhipu_url, &config.zhipu_embedding_model
        );
        let embedding_cli = EmbeddingClient::new(options);
        let chat_cli = ChatClient::from_config(&config);
        let n_results = &config.n_results;
        let batch = config.batch;
        Ok(VectorStore { client, embedding_cli, chat_cli, n_results: n_results.clone(), batch })
    }

    async fn get_collection(
        &self, 
        coll_name: &str, 
        metadata: Option<Map<String, Value>>
    ) -> anyhow::Result<ChromaCollection> {
        self.client.get_or_create_collection(coll_name, metadata).await
    }

    pub async fn list_collections(&self) -> anyhow::Result<Vec<ChromaCollection>> {
        self.client.list_collections().await
    }

    pub async fn delete_collection(
        &self,
        coll_name: &str,
    ) -> anyhow::Result<()> {
        self.client.delete_collection(coll_name).await
            .with_context(|| format!("Cannot remove {}", coll_name))
    }

    pub async fn add(
        &self, 
        coll_name: &str,
        ids: Vec<&str>, 
        documents: Vec<&str>,
        coll_metadata: Option<Map<String, Value>>,
    ) -> anyhow::Result<()> {
        let batch_size = self.batch as usize;
        let mut all_embeddings = Vec::new();

        for chunk in documents.chunks(batch_size) {
            let chunk_vec = chunk.to_vec();
            let embeddings = self.embedding_cli.zhipu_embedding(&chunk_vec).await?
                .iter().map(|embed| embed.embedding.clone()).collect::<Vec<Vec<f32>>>();
            all_embeddings.extend(embeddings);
        }

        let entries = CollectionEntries {
            ids,
            metadatas: None,
            documents: Some(documents),
            embeddings: Some(all_embeddings),
        };
        let collection = self.get_collection(coll_name, coll_metadata).await?;
        collection.upsert(entries, None).await?;
        Ok(())
    }

    pub async fn query_text(
        &self,
        coll_name: &str,
        query_text: Vec<&str>,
    ) -> anyhow::Result<QueryResult> {
        let collection = self.get_collection(coll_name, None).await?;
        let embeddings = self.embedding_cli.zhipu_embedding(&query_text).await?
            .iter().map(|embed| embed.embedding.clone()).collect::<Vec<Vec<f32>>>();
        let query = QueryOptions {
            query_texts: None,
            query_embeddings: Some(embeddings),
            n_results: Some(self.n_results),
            ..Default::default()
        };
        let query_result = collection.query(query, None).await?;
        Ok(query_result)
    }

    pub async fn clean(&self) -> anyhow::Result<()> {
        let collections = self.list_collections().await?;
        let mut iter = collections.iter();
        while let Some(item) = iter.next() {
            self.client.delete_collection(item.name()).await?;
        }
        Ok(())
    }

    pub async fn all_to_differ(&self, coll_name: &str, threshold: f32) -> anyhow::Result<Vec<String>> {
        let collection = self.get_collection(coll_name, None).await?;
        let mut rnt_contexts: HashMap<String, Vec<f32>> = HashMap::new();

        // 分页获取所有的文档ID
        let mut offset: usize = 0;
        let limit: usize = 1000;
        loop {
            let ids = self.get_all_ids(coll_name, offset, limit).await?;
            if ids.is_empty() { break }
            let get_options = GetOptions {
                ids,
                include: Some(vec!["documents".to_string(), "embeddings".to_string()]),
                ..Default::default()
            };
            let result = collection.get(get_options).await?;

            let texts = match result.documents {
                Some(docs) => docs,
                None => return Err(anyhow::anyhow!("Unknown Error")),
            };
            let embeddings = match result.embeddings {
                Some(embeddings) => embeddings,
                None => return Err(anyhow::anyhow!("Unknown Error")),
            };

            let mut result_iter = texts.iter().zip(embeddings);

            while let Some((can_text, can_embed)) = result_iter.next() {
                if let None = can_embed { continue }
                if let None = can_text { continue }
                let can_embed = can_embed.as_ref().unwrap();
                let can_text = can_text.as_ref().unwrap();

                let embeddings_to_compare: Vec<Vec<f32>> = rnt_contexts.values().cloned().collect();
                let mut should_insert = true;
                
                for embedding in &embeddings_to_compare {
                    if self.dissimilarity(can_embed, embedding) < threshold {
                        should_insert = false;
                        break;
                    }
                }
                
                if should_insert {
                    rnt_contexts.insert(can_text.clone(), can_embed.clone());
                }
            }

            offset += limit;
        }
        Ok(rnt_contexts.keys().cloned().collect())
    }

    pub async fn query_embedding(
        &self, 
        coll_name: &str,
        query_embeddings: Vec<Vec<f32>>, 
    ) -> anyhow::Result<QueryResult> {
        let query = QueryOptions {
            query_texts: None,
            query_embeddings: None,
            n_results: None,
            ..Default::default()
        };
        let collection = self.get_collection(coll_name, None).await?;
        let query_result = collection.query(query, None).await?;
        Ok(query_result)
    }    

    // Used ONLY for LLM call
    pub async fn list_collections_llm(&self) -> anyhow::Result<String> {
        let colls = self.list_collections().await?
            .iter().map(|coll| format!("\nname: {}, metadata: {:#?}\n", coll.name(), coll.metadata()))
            .collect::<String>();
        Ok(colls)
    }

    pub async fn query_text_llm(&self, params: HashMap<String, String>) -> anyhow::Result<String> {
       let coll_name = match params.get("collection_name") {
            Some(name) => name,
            None => return Err(anyhow::anyhow!("参数'collection_name'是必要的，表示你需要查找的文本的集合的名称")),
        };

        let text = match params.get("text") {
            Some(t) => t,
            None => return Err(anyhow::anyhow!("参数'text'是必要的，表示你需要查找的文本的内容")),
        };

        let result = self.query_text(
            coll_name.as_str(), vec! [text]
        ).await?;

        if let Some(docs) = result.documents {
            let note = "以下是API输出内容，检查是否包含充足的信息以回答问题，如果不足，请尝试更换关键词继续查询：";
            Ok(docs[0][0].clone() + note)
        } else {
            Ok("None".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::result;

    use super::*;
    use dotenv::dotenv;
    use tokio::fs::read;
    use crate::{chat::{FormatType, Role, Talk}, read_config};

    #[tokio::test]
    async fn test_connect() -> anyhow::Result<()> {
        dotenv().ok();
        let config = read_config()?;
        match VectorStore::from_config(&config).await {
            Ok(_) => {},
            Err(err) => panic!("{}", err.to_string())
        };
        Ok(())
    }

    #[tokio::test]
    async fn test_add() -> anyhow::Result<()> {
        dotenv().ok();
        let config = read_config()?;
        let documents = vec! [
            "helloworld",
            "hellorust",
            "hellochromadb",
        ];
        let ids = vec! [
            "test-1",
            "test-2",
            "test-3",
        ];
        let store = VectorStore::from_config(&config).await?;
        match store.add("test", ids, documents, None).await {
            Ok(_) => Ok(()),
            Err(err) => panic!("{}", err.to_string())
        }
    }   

    #[tokio::test]
    async fn test_list_collection() -> anyhow::Result<()> {
        let config = read_config()?;
        let store = VectorStore::from_config(&config).await?;
        println!("LLM sees: {}", store.list_collections_llm().await?);
        Ok(())
    }

    #[tokio::test]
    async fn test_query() {
        let config = read_config().unwrap();
        let store = VectorStore::from_config(&config).await.unwrap();
        let result = store.query_text("coffee", vec!["咖啡水洗处理法"]).await.unwrap();
        println!("{:?}", result.documents)
    }

    #[tokio::test]
    async fn test_whole_query() {
        let config = read_config().unwrap();
        let store = VectorStore::from_config(&config).await.unwrap();
        let cli = ChatClient::from_config(&config);
        let mut msgs: Vec<Talk> = Vec::new();
        msgs.push(Talk::new(Role::System, "你的工作是利用给定的文段总结出关于半导体芯片的
            细分行业".to_string()));
        let result = match store.all_to_differ("semiconductor-chip", 0.4).await {
            Ok(res) => res,
            Err(err) => panic!("{}", err.to_string()),
        };
        println!("{:?}", result);
        
        msgs.push(Talk::new(Role::User, format!("{:?}", result)));
        cli.get_completion(&mut msgs, FormatType::Text).await.unwrap();
        
        println!("{:?}", msgs.last());
    }
}
