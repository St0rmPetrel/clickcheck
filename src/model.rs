use clap::ValueEnum;
use clickhouse::Row;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use time::OffsetDateTime;

use crate::cli;

#[derive(Row, Serialize, Deserialize, Debug, Clone)]
pub struct QueryLog {
    // Базовые метрики (raw values)
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
    // Композитные показатели
    pub io_impact: u64,     // Специализированный I/O вес
    pub cpu_impact: u64,    // Специализированный CPU вес
    pub memory_impact: u64, // Специализированный memory вес
    pub time_impact: u64,   // Специализированный latency вес
    pub total_impact: u64,  // Основной агрегированный показатель
}

#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    Json,
    Yaml,
    Text,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum QueriesSortBy {
    TotalImpact,
    IOImpact,
    CPUImpact,
    MemoryImpact,
    TimeImpact,
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
    pub last: Option<Duration>,
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
            last: args.last,
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
    #[serde(skip)]
    pub password: String,
    pub urls: Vec<String>,
    pub accept_invalid_certificate: bool,
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
