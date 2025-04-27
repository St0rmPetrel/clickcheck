use crate::model::QueryLog;
use clickhouse::{Client as ChClient, error::Error as ChError};
use thiserror::Error;
use tokio::sync::mpsc::{Sender, error::SendError};

pub struct Client(ChClient);

pub struct Config<'a> {
    pub url: &'a str,
    pub user: &'a str,
    pub password: &'a str,
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("clickhouse query error: {0}")]
    Query(#[from] ChError),

    #[error("failed to send query log to analyzer: {0}")]
    Send(#[from] SendError<QueryLog>),
}

impl Client {
    pub fn new(cfg: Config) -> Self {
        let inner = ChClient::default()
            .with_url(cfg.url)
            .with_user(cfg.user)
            .with_password(cfg.password)
            .with_database("system");

        Self(inner)
    }

    pub async fn stream_query_logs(&self, sender: Sender<QueryLog>) -> Result<(), ClientError> {
        let mut cursor = self
            .0
            .query(
                r#"
                SELECT
                    any(query) AS query,
                    max(event_time) AS max_event_time,
                    min(event_time) AS min_event_time,
                    sum(query_duration_ms) AS query_duration_ms,
                    sum(read_rows) AS read_rows,
                    sum(read_bytes) AS read_bytes,
                    sum(memory_usage) AS memory_usage,
                    sum(ProfileEvents['UserTimeMicroseconds']) AS user_time_us,
                    sum(ProfileEvents['SystemTimeMicroseconds']) AS system_time_us
                FROM query_log
                WHERE type != 'QueryStart'
                  AND query_kind = 'Select'
                GROUP BY normalized_query_hash
                "#,
            )
            .fetch::<QueryLog>()?;

        while let Some(row) = cursor.next().await? {
            sender.send(row).await?
        }

        Ok(())
    }
}
