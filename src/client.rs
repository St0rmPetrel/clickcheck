mod filter;

use crate::model::{Error, QueryLog};
use clickhouse::{Client as ChClient, error::Error as ChError};
use futures::future::try_join_all;
use thiserror::Error;
use tokio::sync::mpsc::{Sender, error::SendError};

use filter::{ErrorFilter, QueryLogFilter};

pub struct Client {
    nodes: Vec<ChClient>,
}

pub struct Config<'a> {
    pub urls: &'a [String],
    pub user: &'a str,
    pub password: &'a str,
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("clickhouse query error: {0}")]
    Query(#[from] ChError),

    #[error("failed to send query log: {0}")]
    SendQueryLog(#[from] SendError<QueryLog>),

    #[error("failed to send error: {0}")]
    SendError(#[from] SendError<Error>),

    #[error("failed to format datetime for query: {0}")]
    Format(#[from] time::error::Format),
}

impl Client {
    pub fn new(cfg: Config) -> Self {
        let nodes = cfg
            .urls
            .iter()
            .map(|url| {
                ChClient::default()
                    .with_url(url)
                    .with_user(cfg.user)
                    .with_password(cfg.password)
                    .with_database("system")
            })
            .collect();

        Self { nodes }
    }

    pub async fn stream_logs_by_fingerprint(
        &self,
        filter: QueryLogFilter,
        sender: Sender<QueryLog>,
    ) -> Result<(), ClientError> {
        let mut futures = Vec::new();

        for node in &self.nodes {
            let sender = sender.clone();
            let node = node.clone();
            let filter = filter.clone();

            let fut = async move {
                let (where_clause, params) = filter.build_where();
                let query = format!(
                    r#"
                        SELECT
                            normalized_query_hash,
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
                          {}
                        GROUP BY normalized_query_hash
                        "#,
                    &where_clause,
                );

                let mut query = node.query(&query);
                for param in params {
                    query = query.bind(param.to_sql_string()?);
                }

                let mut cursor = query.fetch::<QueryLog>()?;

                while let Some(row) = cursor.next().await? {
                    sender.send(row).await?;
                }

                Ok::<(), ClientError>(())
            };

            futures.push(fut);
        }

        try_join_all(futures).await?;

        Ok(())
    }

    pub async fn stream_error_by_code(
        &self,
        filter: ErrorFilter,
        sender: Sender<Error>,
    ) -> Result<(), ClientError> {
        let mut futures = Vec::new();

        for node in &self.nodes {
            let sender = sender.clone();
            let node = node.clone();
            let filter = filter.clone();

            let fut = async move {
                let (where_clause, where_params) = filter.build_where();
                let (having_clause, having_params) = filter.build_having();
                let query = format!(
                    r#"
                        SELECT
                            code,
                            any(name)        AS name,
                            sum(value)       AS count,
                            max(last_error_time)    AS last_error_time,
                            any(last_error_message) AS error_message
                        FROM system.errors
                        WHERE 1 = 1
                          {}
                        GROUP BY code
                        HAVING 1 = 1
                          {}
                        "#,
                    &where_clause, &having_clause,
                );

                // Combine all parameters
                let mut params = Vec::new();
                params.extend(where_params);
                params.extend(having_params);

                let mut query = node.query(&query);
                for param in params {
                    query = query.bind(param.to_sql_string()?);
                }

                let mut cursor = query.fetch::<Error>()?;

                while let Some(row) = cursor.next().await? {
                    sender.send(row).await?;
                }

                Ok::<(), ClientError>(())
            };

            futures.push(fut);
        }

        try_join_all(futures).await?;

        Ok(())
    }
}
