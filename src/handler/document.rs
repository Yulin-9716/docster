use std::path::PathBuf;
use walkdir::DirEntry;
use crate::document::{process_document, chunk::chunk_document};
use crate::vector_store::VectorStore;

pub async fn add_documents(
    store: &VectorStore,
    path: PathBuf,
    name: &str,
    recursive: bool,
    chunk_size: usize
) -> anyhow::Result<()> {
    println!("正在处理文档: {}", path.display());

    if path.is_dir() {
        process_directory(store, path, name, recursive, chunk_size).await
    } else {
        process_single_file(store, path, name, chunk_size).await
    }
}

async fn process_directory(
    store: &VectorStore,
    path: PathBuf,
    name: &str,
    recursive: bool,
    chunk_size: usize
) -> anyhow::Result<()> {
    let entries = get_entries(&path, recursive);
    
    for entry in entries {
        let entry_path = entry.path();
        if entry_path.is_file() {
            if let Some(ext) = entry_path.extension().and_then(|e| e.to_str()) {
                match ext {
                    "pdf" | "docx" => {
                        process_single_file(store, entry_path.to_path_buf(), name, chunk_size).await?;
                    },
                    _ => println!("警告: 跳过不支持的文件类型: {}", entry_path.display()),
                }
            }
        }
    }
    Ok(())
}

async fn process_single_file(
    store: &VectorStore,
    path: PathBuf,
    name: &str,
    chunk_size: usize
) -> anyhow::Result<()> {
    let file_stem = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    
    println!("正在处理文档: {}", path.display());
    
    let content = process_document(&path)?;
    let chunks = chunk_document(content, chunk_size);

    println!("文档 {} 切块完成, 共分成{}块", path.display(), chunks.len());
    
    let mut doc_ids = Vec::new();
    let mut texts = Vec::new();
    
    for (i, chunk) in chunks.into_iter().enumerate() {
        doc_ids.push(format!("{}-{}", file_stem, i));
        texts.push(chunk.content);
    }
    
    store.add(
        name,
        doc_ids.iter().map(|s| s.as_str()).collect(),
        texts.iter().map(|s| s.as_str()).collect(),
        None,
    ).await?;

    Ok(())
}

fn get_entries(path: &PathBuf, recursive: bool) -> Box<dyn Iterator<Item = DirEntry>> {
    let iter = if recursive {
        walkdir::WalkDir::new(path)
    } else {
        walkdir::WalkDir::new(path).max_depth(1)
    };
    Box::new(iter.into_iter().filter_map(|e| e.ok()))
}

pub async fn list_collections(store: &VectorStore) -> anyhow::Result<()> {
    let collections = store.list_collections().await?;
    println!("\n所有的集合：");
    for coll in collections {
        println!("\t{}", coll.name());
    }
    Ok(())
}

pub async fn remove_collection(store: &VectorStore, name: &str) -> anyhow::Result<()> {
    store.delete_collection(name).await?;
    Ok(())
}

pub async fn clean_collections(store: &VectorStore) -> anyhow::Result<()> {
    store.clean().await?;
    println!("已清空");
    Ok(())
}
