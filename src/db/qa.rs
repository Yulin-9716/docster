use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct QA {
    pub id: i32,
    pub question: String,
    pub answer: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl QA {
    pub async fn create(pool: &Pool, question: &str, answer: &str) -> anyhow::Result<QA> {
        let client = pool.get().await?;
        let stmt = client.prepare(
            "INSERT INTO qa_pairs (question, answer) VALUES ($1, $2) RETURNING *"
        ).await?;
        let row = client.query_one(&stmt, &[&question, &answer]).await?;
        QA::from_row(&row).map_err(Into::into)
    }

    pub async fn get_by_id(pool: &Pool, id: i32) -> anyhow::Result<Option<QA>> {
        let client = pool.get().await?;
        let stmt = client.prepare("SELECT * FROM qa_pairs WHERE id = $1").await?;
        if let Some(row) = client.query_opt(&stmt, &[&id]).await? {
            Ok(Some(QA::from_row(&row)?))
        } else {
            Ok(None)
        }
    }

    pub async fn update(pool: &Pool, id: i32, question: &str, answer: &str) -> anyhow::Result<QA> {
        let client = pool.get().await?;
        let stmt = client.prepare(
            "UPDATE qa_pairs SET question = $1, answer = $2 WHERE id = $3 RETURNING *"
        ).await?;
        let row = client.query_one(&stmt, &[&question, &answer, &id]).await?;
        QA::from_row(&row).map_err(Into::into)
    }

    pub async fn delete(pool: &Pool, id: i32) -> anyhow::Result<()> {
        let client = pool.get().await?;
        let stmt = client.prepare("DELETE FROM qa_pairs WHERE id = $1").await?;
        client.execute(&stmt, &[&id]).await?;
        Ok(())
    }

    pub async fn list_all(pool: &Pool) -> anyhow::Result<Vec<QA>> {
        let client = pool.get().await?;
        let stmt = client.prepare("SELECT * FROM qa_pairs ORDER BY created_at DESC").await?;
        let rows = client.query(&stmt, &[]).await?;
        rows.iter().map(|row| QA::from_row(row)).collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    fn from_row(row: &tokio_postgres::Row) -> anyhow::Result<QA> {
        Ok(QA {
            id: row.get("id"),
            question: row.get("question"),
            answer: row.get("answer"),
            created_at: row.try_get::<_, chrono::NaiveDateTime>("created_at")?.and_utc(),
        })
    }
}
