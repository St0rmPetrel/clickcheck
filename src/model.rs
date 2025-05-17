use clap::ValueEnum;
use clickhouse::Row;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use time::OffsetDateTime;
use std::time::Duration;

use crate::cli;

#[derive(Row, Serialize, Deserialize, Debug, Clone)]
pub struct QueryLog {
    pub normalized_query_hash: u64,
    pub query: String,
    #[serde(with = "clickhouse::serde::time::datetime")]
    pub max_event_time: OffsetDateTime,
    #[serde(with = "clickhouse::serde::time::datetime")]
    pub min_event_time: OffsetDateTime,
    pub query_duration_ms: u64,
    pub read_rows: u64,
    pub read_bytes: u64,
    pub memory_usage: u64,
    pub user_time_us: u64,
    pub system_time_us: u64,
}

impl QueryLog {
    /// Целочисленный вес запроса, без `f64`:
    ///
    /// weight = cpu_time_us*10_000
    ///        + memory_usage*10
    ///        + query_duration_ms*1_000_000
    ///        + read_rows*100
    ///        + read_bytes*1
    pub fn weight(&self) -> u64 {
        // Суммарное CPU-время в микросекундах
        let cpu_time_us = self.user_time_us + self.system_time_us;

        // Целочисленные коэффициенты
        const W_CPU: u64 = 10_000;
        const W_MEM: u64 = 10;
        const W_DUR: u64 = 1_000_000;
        const W_ROWS: u64 = 100;
        const W_BYTES: u64 = 1;

        cpu_time_us
            .saturating_mul(W_CPU)
            .saturating_add(self.memory_usage.saturating_mul(W_MEM))
            .saturating_add(self.query_duration_ms.saturating_mul(W_DUR))
            .saturating_add(self.read_rows.saturating_mul(W_ROWS))
            .saturating_add(self.read_bytes.saturating_mul(W_BYTES))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WeightedQueryLog {
    pub weight: u64,
    pub total_weight: u64,
    pub cpu_time_us: u64,
    pub query: QueryLog,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    Json,
    Yaml,
    Text,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum QueriesSortBy {
    Weight,
    CpuTime,
    QueryDuration,
    ReadRows,
    ReadBytes,
    MemoryUsage,
    UserTime,
    SystemTime,
}

#[derive(Debug)]
pub struct QueriesFilter {
    pub from: Option<OffsetDateTime>,
    pub to: Option<OffsetDateTime>,
}

#[derive(Debug)]
pub struct TopQueriesRequest {
    pub limit: usize,
    pub sort_by: QueriesSortBy,
    pub filter: QueriesFilter,
    pub out: OutputFormat,
}

#[derive(Clone, Debug)]
pub struct ErrorsFilter {
    pub last: Option<Duration>,
    pub min_count: Option<usize>,
    pub code: Vec<i32>,
}

#[derive(Debug)]
pub struct TopErrorsRequest {
    pub limit: usize,
    pub filter: ErrorsFilter,
    pub out: OutputFormat,
}

impl From<cli::QueriesFilterArgs> for QueriesFilter {
    fn from(args: cli::QueriesFilterArgs) -> Self {
        Self {
            from: args.from,
            to: args.to,
        }
    }
}

impl From<cli::ErrorFilterArgs> for ErrorsFilter {
    fn from(args: cli::ErrorFilterArgs) -> Self {
        Self {
            last: args.last,
            min_count: args.min_count,
            code: args.code,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContextProfile {
    pub user: String,
    pub password: String,
    pub urls: Vec<String>,
    pub accept_invalid_certificate: Option<bool>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ContextConfig {
    pub current: Option<String>,
    pub profiles: HashMap<String, ContextProfile>,
}

#[derive(Debug)]
pub struct ContextSetProfileRequest {
    pub name: String,
    pub user: String,
    pub password: String,
    pub urls: Vec<String>,
}

#[derive(Row, Serialize, Deserialize, Debug, Clone)]
/// Contains a list of all errors which have ever happened
/// including the error code, last time and
/// message with unsymbolized stacktrace.
pub struct Error {
    pub code: i32,
    pub name: String,
    pub count: u64,
    #[serde(with = "clickhouse::serde::time::datetime")]
    pub last_error_time: OffsetDateTime,
    pub error_message: String,
}
