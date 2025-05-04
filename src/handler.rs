use std::path::PathBuf;
use clap::{command, Parser};
use crate::Config;

mod document;
mod query;
mod write;

#[derive(Parser)]
#[command(name = "graph-rag")]
#[command(about = "基于GraphRAG的知识库问答系统", version = "1.0")]
pub enum Cli {
    /// 文档管理命令
    #[command(subcommand)]
    Doc(DocCommand),

    /// 启动问答会话
    Chat {
        #[arg(long, help = "保存会话记录", default_value_t = true)]
        save: bool,
    },

    /// 导出问答数据到Excel
    Write {
        #[arg(long, help = "输出文件路径")]
        path: PathBuf,
    },
}

#[derive(Parser)]
pub enum DocCommand {
    /// 上传文档到知识库
    Add {
        #[arg(help = "文件或目录路径")]
        path: PathBuf,

        #[arg(short, long, help = "递归处理子目录")]
        recursive: bool,

        #[arg(short, long, help = "添加的集合名称")]
        name: String,
    },

    List,

    Remove {
        #[arg(help = "集合名称")]
        name: String,
    },

    Clean,
}

pub async fn handler(args: Cli, config: Config, pool: deadpool_postgres::Pool) -> anyhow::Result<()> {
    match args {
        Cli::Doc(cmd) => handle_doc_command(cmd, config).await,
        Cli::Chat { save } => query::handle_query_session(&pool, config, save).await,
        Cli::Write { path } => write::export_to_excel(&pool, path).await,
    }
}

async fn handle_doc_command(cmd: DocCommand, config: Config) -> anyhow::Result<()> {
    use document::*;
    let store = crate::vector_store::VectorStore::from_config(&config).await?;

    match cmd {
        DocCommand::Add { path, name, recursive } => {
            add_documents(&store, path, &name, recursive, config.chunk_size as usize).await
        }
        DocCommand::List => list_collections(&store).await,
        DocCommand::Remove { name } => remove_collection(&store, &name).await,
        DocCommand::Clean => clean_collections(&store).await,
    }
}
