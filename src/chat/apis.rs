use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result as FmtResult};

use crate::vector_store::VectorStore;

pub struct API {
    pub name: String,
    pub description: String,
    pub parameters: Option<Vec<String>>,
}

pub struct Registry {
    pub apis: Vec<API>,
}

impl Display for API {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if let Some(params) = &self.parameters {
            write!(
                f,
                "API: {}\nDescription: {}\nParameters: {:?}\n",
                self.name,
                self.description,
                params,
            )
        } else {
            write!(
                f,
                "API: {}\nDescription: {}\nParameters: None\n",
                self.name,
                self.description,
            )
        }
    }
}

impl Registry {
    pub fn new() -> Self {
        Self {
            apis: vec![
                API {
                    name: "list_collection".to_string(),
                    description: "列出所有的集合的名称".to_string(),
                    parameters: Some(vec![]),
                },
                API {
                    name: "query".to_string(),
                    description: "从指定名称的集合中查询相关信息。collection_name表示名称，text表示要查询的内容".to_string(),
                    parameters: Some(vec![
                        "collection_name".to_string(), 
                        "text".to_string(),
                    ]),
                },
            ]
        }
    }

    pub fn list_apis(&self) -> String {
        let mut info = String::new();
        let mut iter = self.apis.iter();
        while let Some(api) = iter.next() {
            info.push_str(api.to_string().as_str());
            info.push_str("\n");
        }
        info
    }

    pub async fn handle(&self, store: &VectorStore, func_name: String, params: HashMap<String, String>)
     -> anyhow::Result<String> {
        match func_name.as_str() {
            "list_collection" => store.list_collections_llm().await,
            "query" => store.query_text_llm(params).await,
            other => Err(anyhow::anyhow!("No such API: {}", other)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apis() -> anyhow::Result<()> {
        let registry = Registry::new();
        let api_infos = registry.list_apis();
        println!("{}", api_infos);
        Ok(())
    }
}
 
