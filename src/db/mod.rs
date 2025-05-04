use deadpool_postgres::{Manager, Pool};
use tokio_postgres::Config;

pub mod qa;
pub use qa::QA;

pub async fn create_pool() -> anyhow::Result<Pool> {
    let mut cfg = Config::new();
    cfg.host("localhost");
    cfg.user("postgres");
    cfg.password("postgres");
    cfg.dbname("docster");
    
    let mgr = Manager::new(cfg, tokio_postgres::NoTls);
    let pool = Pool::builder(mgr).max_size(16).build()?;
    Ok(pool)
}
