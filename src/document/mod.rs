pub mod pdf;
pub mod docx;
pub mod chunk;

use std::path::{Path, PathBuf};
use std::collections::HashMap;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub path: PathBuf,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DocumentList {
    documents: Vec<DocumentMetadata>,
}

pub fn process_document(path: &Path) -> Result<String> {
    let content = match path.extension().and_then(|s| s.to_str()) {
        Some("pdf") => pdf::extract(path)?,
        Some("docx") => docx::extract(path)?,
        _ => anyhow::bail!("目前仅支持PDF和DOCX文件"),
    };
    
    let processed = content.replace(|c: char| c.is_control(), "")
        .replace("。", ".")
        .replace("，", ",");
        
    Ok(processed)
}
