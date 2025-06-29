//! ClickHouse client abstraction with streaming and filtering support.
//!
//! This module defines the [`Client`] struct, which wraps multiple ClickHouse nodes
//! and provides methods to stream query logs and error metrics based on user-defined filters.
//!
//! ## Responsibilities
//! - Connecting to ClickHouse nodes securely or with relaxed TLS.
//! - Executing queries concurrently across multiple nodes.
//! - Building dynamic SQL queries from filter parameters.
//! - Sending results through async channels.
//!
//! ## Key Types
//! - [`Client`] — Manages ClickHouse node connections and runs filtered queries.
//! - [`Config`] — Holds credentials and configuration for instantiating [`Client`].
//! - [`ClientError`] — Defines errors returned from client operations.
//!
//! ## Supported Operations
//! - [`Client::stream_logs_by_fingerprint`] — Streams normalized query log summaries.
//! - [`Client::stream_error_by_code`] — Streams frequent ClickHouse errors grouped by code.
//!
//! ## Filtering
//! Query filtering is handled internally and supports filtering query logs and system errors.
//!
//! ## TLS
//! If `danger_accept_invalid_certs` is true in [`Config`], the client will
//! accept invalid or self-signed certificates (intended for dev/test environments).
//!
//! ## Notes
//! - Uses custom impact score formulas (I/O, CPU, memory, time) to rank query logs.
//! - Designed for async environments using [`tokio`] and channels.
//! - Includes fallbacks for secure and insecure HTTP client construction.
//!
//! This module forms the core data access layer for ClickHouse-backed analytics.

mod filter;

use crate::model::{Error, QueryLog, QueryLogExtended, QueryLogTotal};
use clickhouse::{error::Error as ChError, query::Query as ChQuery, Client as ChClient, Row};
use filter::{ErrorFilter, QueryLogFilter};
use futures::future::try_join_all;
use hyper_tls::native_tls;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client as HyperClient;
use hyper_util::rt::TokioExecutor;
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;
use tokio::sync::mpsc::{error::SendError, Sender};

// Константы для конфигурации HTTP клиента
const TCP_KEEPALIVE: Duration = Duration::from_secs(60);
// ClickHouse uses 3s by default.
// See https://github.com/ClickHouse/ClickHouse/blob/368cb74b4d222dc5472a7f2177f6bb154ebae07a/programs/server/config.xml#L201
const POOL_IDLE_TIMEOUT: Duration = Duration::from_secs(2);

pub struct Client {
    nodes: Vec<ChClient>,
}

pub struct Config<'a> {
    pub urls: &'a [String],
    pub user: &'a str,
    pub password: &'a secrecy::SecretString,
    pub danger_accept_invalid_certs: bool,
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("clickhouse query error: {0}")]
    Query(#[from] ChError),

    #[error("failed to send query log: {0}")]
    SendQueryLog(#[from] SendError<QueryLog>),

    #[error("failed to send error: {0}")]
    SendError(#[from] SendError<Error>),

    #[error("channel send failure")]
    Send,

    #[error("failed to format datetime for query: {0}")]
    Format(#[from] time::error::Format),

    #[error("failed to create native tls config: {0}")]
    InitializationError(#[from] native_tls::Error),
}

fn from_insecure_hyper_client() -> Result<ChClient, ClientError> {
    let mut connector = HttpConnector::new(); // or HttpsConnectorBuilder

    connector.set_keepalive(Some(TCP_KEEPALIVE));
    connector.enforce_http(false);

    let tls = native_tls::TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    let connector = hyper_tls::HttpsConnector::from((connector, tls.into()));

    let hyper_client = HyperClient::builder(TokioExecutor::new())
        .pool_idle_timeout(POOL_IDLE_TIMEOUT)
        .build(connector);

    Ok(ChClient::with_http_client(hyper_client))
}

impl Client {
    /// Creates a new `Client` instance that connects to the provided ClickHouse nodes.
    ///
    /// # Arguments
    ///
    /// * `cfg` - Configuration with URLs, credentials, and TLS settings.
    ///
    /// # Errors
    ///
    /// Returns `ClientError` if initialization fails, including problems with TLS or URL handling.
    pub fn new(cfg: Config) -> Result<Self, ClientError> {
        let nodes = cfg
            .urls
            .iter()
            .map(|url| {
                let node = if cfg.danger_accept_invalid_certs {
                    from_insecure_hyper_client()?
                } else {
                    ChClient::default()
                }
                .with_url(url)
                .with_user(cfg.user)
                .with_password(cfg.password.expose_secret())
                .with_database("system");
                Ok::<ChClient, ClientError>(node)
            })
            .collect::<Result<Vec<_>, ClientError>>()?;

        Ok(Self { nodes })
    }

    async fn execute_on_all_nodes<R, B>(
        &self,
        sender: Sender<R>,
        build_query: B,
    ) -> Result<(), ClientError>
    where
        R: Serialize + Row + Send + Deserialize<'static> + 'static,
        B: Fn(&ChClient) -> Result<ChQuery, ClientError> + Send + Sync + 'static + Clone,
    {
        let futures = self.nodes.iter().map(|node| {
            let build_query = build_query.clone();
            let sender = sender.clone();
            let node = node.clone();

            async move {
                let q = build_query(&node)?;
                let mut cursor = q.fetch::<R>()?;

                while let Some(row) = cursor.next().await? {
                    sender.send(row).await.map_err(|_| ClientError::Send)?;
                }

                Ok::<(), ClientError>(())
            }
        });

        try_join_all(futures).await?;
        Ok(())
    }

    /// Streams grouped query log data matching the specified filter, grouped by fingerprint (`normalized_query_hash`).
    ///
    /// Useful for identifying query patterns and their cumulative impact across the system.
    ///
    /// # Arguments
    ///
    /// * `filter` - Filter criteria (time range, user, etc.).
    /// * `sender` - A `Sender<QueryLog>` to push results into a stream or channel.
    ///
    /// # Errors
    ///
    /// Returns `ClientError` if the query fails, parsing fails, or sending on the channel fails.
    ///
    /// # ClickHouse schema dependency
    ///
    /// This relies on the `system.query_log` table and expects ClickHouse to be configured to log queries.
    pub async fn stream_logs_by_fingerprint(
        &self,
        filter: QueryLogFilter,
        sender: Sender<QueryLog>,
    ) -> Result<(), ClientError> {
        let (where_clause, where_params) = filter.build_where();
        let sql = format!(
            r#"
            WITH
               sum(query_duration_ms) AS total_query_duration_ms,
               sum(read_rows) AS total_read_rows,
               sum(read_bytes) AS total_read_bytes,
               sum(memory_usage) AS total_memory_usage,
               sum(ProfileEvents['UserTimeMicroseconds']) AS total_user_time_us,
               sum(ProfileEvents['SystemTimeMicroseconds']) AS total_system_time_us,
               sum(ProfileEvents['NetworkReceiveBytes']) AS total_network_receive_bytes,
               sum(ProfileEvents['NetworkSendBytes']) AS total_network_send_bytes
            SELECT
               normalized_query_hash,
               any(query) AS query,
               total_read_rows * 100 + total_read_bytes * 1 AS io_impact,
               total_network_receive_bytes * 10 + total_network_send_bytes * 10 AS network_impact,
               total_user_time_us * 10_000 + total_system_time_us * 10_000 AS cpu_impact,
               total_memory_usage * 10 AS memory_impact,
               total_query_duration_ms * 1_000_000 AS time_impact,
               io_impact + network_impact + cpu_impact + memory_impact + time_impact AS total_impact
            FROM query_log
            WHERE type != 'QueryStart' AND query_kind = 'Select' {where_clause}
            GROUP BY normalized_query_hash
            "#,
        );

        self.execute_on_all_nodes(sender, move |node| {
            build_query_with_params(node, &sql, &where_params)
        })
        .await
    }

    /// Retrieves detailed query log metrics for a specific query fingerprint.
    ///
    /// This method is intended for drill-down analysis of a known query fingerprint
    /// (`normalized_query_hash`). Unlike [`Self::stream_logs_by_fingerprint`],
    /// which aggregates metrics across *all* fingerprints, this function focuses on a
    /// single query group, providing extended information.
    ///
    /// # Arguments
    ///
    /// * `fingerprint` — The `normalized_query_hash` of the query group to inspect.
    /// * `filter` — Optional additional filtering (e.g., time range, user).
    /// * `sender` — A `Sender<QueryLogExtended>` to stream the result.
    ///
    /// # Returns
    ///
    /// * `Ok(())` — If the query completed successfully.
    /// * `Err(ClientError)` — On ClickHouse query failure or channel send error.
    ///
    /// # ClickHouse schema dependency
    ///
    /// Relies on the `system.query_log` table and assumes it includes normalized query hashes.
    pub async fn stream_log_by_fingerprint(
        &self,
        fingerprint: u64,
        filter: QueryLogFilter,
        sender: Sender<QueryLogExtended>,
    ) -> Result<(), ClientError> {
        let (where_clause, where_params) = filter.build_where();

        let sql = format!(
            r#"
            SELECT
               normalized_query_hash,
               any(query) AS query,
               max(event_time) AS max_event_time,
               min(event_time) AS min_event_time,
               sum(query_duration_ms) AS total_query_duration_ms,
               sum(read_rows) AS total_read_rows,
               sum(read_bytes) AS total_read_bytes,
               sum(memory_usage) AS total_memory_usage,
               sum(ProfileEvents['UserTimeMicroseconds']) AS total_user_time_us,
               sum(ProfileEvents['SystemTimeMicroseconds']) AS total_system_time_us,
               sum(ProfileEvents['NetworkReceiveBytes']) AS total_network_receive_bytes,
               sum(ProfileEvents['NetworkSendBytes']) AS total_network_send_bytes,
               groupUniqArray(user) AS users,
               arrayDistinct(arrayFlatten(groupArray(databases))) AS databases,
               arrayDistinct(arrayFlatten(groupArray(tables))) AS tables
            FROM query_log
            WHERE type != 'QueryStart' AND query_kind = 'Select'
              AND normalized_query_hash = {fingerprint} {where_clause}
            GROUP BY normalized_query_hash
            "#,
        );

        self.execute_on_all_nodes(sender, move |node| {
            build_query_with_params(node, &sql, &where_params)
        })
        .await
    }

    /// Streams total aggregated query log metrics matching the specified filter.
    ///
    /// Unlike [`Self::stream_logs_by_fingerprint`], this method does not group by query fingerprint.
    /// Instead, it aggregates metrics across all matching `Select` queries within the `query_log`
    /// for a given filter, returning a single total result.
    ///
    /// Useful for high-level monitoring of cluster-wide query impact over a period of time.
    ///
    /// # Arguments
    ///
    /// * `filter` - Filter criteria to restrict the aggregation scope (e.g., time range, user).
    /// * `sender` - A `Sender<QueryLogTotal>` to deliver the aggregated result to consumers.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the aggregation completed successfully.
    /// * `Err(ClientError)` - On query failure, data parsing issue, or channel send error.
    ///
    /// # ClickHouse schema dependency
    ///
    /// Relies on the `system.query_log` table with query profiling enabled.
    pub async fn stream_logs_total(
        &self,
        filter: QueryLogFilter,
        sender: Sender<QueryLogTotal>,
    ) -> Result<(), ClientError> {
        let (where_clause, where_params) = filter.build_where();
        let sql = format!(
            r#"
            WITH
               sum(query_duration_ms) AS total_query_duration_ms,
               sum(read_rows) AS total_read_rows,
               sum(read_bytes) AS total_read_bytes,
               sum(memory_usage) AS total_memory_usage,
               sum(ProfileEvents['UserTimeMicroseconds']) AS total_user_time_us,
               sum(ProfileEvents['SystemTimeMicroseconds']) AS total_system_time_us,
               sum(ProfileEvents['NetworkReceiveBytes']) AS total_network_receive_bytes,
               sum(ProfileEvents['NetworkSendBytes']) AS total_network_send_bytes
            SELECT
               count() AS queries_count,
               total_read_rows * 100 + total_read_bytes * 1 AS io_impact,
               total_network_receive_bytes * 10 + total_network_send_bytes * 10 AS network_impact,
               total_user_time_us * 10_000 + total_system_time_us * 10_000 AS cpu_impact,
               total_memory_usage * 10 AS memory_impact,
               total_query_duration_ms * 1_000_000 AS time_impact,
               io_impact + network_impact + cpu_impact + memory_impact + time_impact AS total_impact
            FROM query_log
            WHERE type != 'QueryStart' AND query_kind = 'Select' {where_clause}
            "#,
        );

        self.execute_on_all_nodes(sender, move |node| {
            build_query_with_params(node, &sql, &where_params)
        })
        .await
    }

    /// Streams error statistics from ClickHouse's `system.errors` table based on the provided filter.
    ///
    /// Useful for monitoring the frequency and severity of runtime errors by error code.
    ///
    /// # Arguments
    ///
    /// * `filter` - Filter criteria (e.g. time range, minimum count).
    /// * `sender` - A `Sender<Error>` to stream the results.
    ///
    /// # Errors
    ///
    /// Returns `ClientError` for query or channel failures.
    ///
    /// # ClickHouse schema dependency
    ///
    /// Requires that `system.errors` is populated (i.e., error collection is enabled in ClickHouse).
    pub async fn stream_error_by_code(
        &self,
        filter: ErrorFilter,
        sender: Sender<Error>,
    ) -> Result<(), ClientError> {
        let (where_clause, where_params) = filter.build_where();
        let (having_clause, having_params) = filter.build_having();
        let sql = format!(
            r#"
            SELECT
                code,
                any(name)        AS name,
                sum(value)       AS count,
                max(last_error_time)    AS last_error_time,
                any(last_error_message) AS error_message
            FROM system.errors
            WHERE 1 = 1
              {where_clause}
            GROUP BY code
            HAVING 1 = 1
              {having_clause}
            "#,
        );
        let params = [where_params, having_params].concat();

        self.execute_on_all_nodes(sender, move |node| {
            build_query_with_params(node, &sql, &params)
        })
        .await
    }
}

fn build_query_with_params(
    node: &ChClient,
    sql: &str,
    params: &[filter::QueryParam],
) -> Result<ChQuery, ClientError> {
    let mut query = node.query(sql);
    for param in params {
        query = query.bind(param.to_sql_string()?);
    }
    Ok(query)
}
