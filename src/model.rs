use clap::ValueEnum;
use clickhouse::Row;
use secrecy::ExposeSecret;
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
    pub total_query_duration_ms: u64,
    pub total_read_rows: u64,
    pub total_read_bytes: u64,
    pub total_memory_usage: u64,
    pub total_user_time_us: u64,
    pub total_system_time_us: u64,
    pub total_network_receive_bytes: u64,
    pub total_network_send_bytes: u64,
    pub users: Vec<String>,
    pub databases: Vec<String>,
    pub tables: Vec<String>,
    // Композитные показатели
    pub io_impact: u64,      // Специализированный I/O вес
    pub network_impact: u64, // Специализированный Network вес
    pub cpu_impact: u64,     // Специализированный CPU вес
    pub memory_impact: u64,  // Специализированный memory вес
    pub time_impact: u64,    // Специализированный latency вес
    pub total_impact: u64,   // Основной агрегированный показатель
}

#[derive(Row, Serialize, Deserialize, Debug, Clone, Default)]
pub struct QueryLogTotal {
    pub queries_count: u64, // Количество Select запросов
    // Композитные показатели
    pub io_impact: u64,      // Специализированный I/O вес
    pub network_impact: u64, // Специализированный Network вес
    pub cpu_impact: u64,     // Специализированный CPU вес
    pub memory_impact: u64,  // Специализированный memory вес
    pub time_impact: u64,    // Специализированный latency вес
    pub total_impact: u64,   // Основной агрегированный показатель
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
    NetworkImpact,
}

#[derive(Debug)]
pub struct QueriesFilter {
    pub from: Option<OffsetDateTime>,
    pub to: Option<OffsetDateTime>,
    pub last: Option<Duration>,
    pub users: Vec<String>,
    pub databases: Vec<String>,
    pub tables: Vec<String>,
    pub min_query_duration: Option<std::time::Duration>,
    pub min_read_rows: Option<u64>,
    pub min_read_data: Option<bytesize::ByteSize>,
}

#[derive(Debug)]
pub struct TopQueriesRequest {
    pub limit: usize,
    pub sort_by: QueriesSortBy,
    pub filter: QueriesFilter,
    pub out: OutputFormat,
}

#[derive(Debug)]
pub struct TotalQueriesRequest {
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
            users: args.query_user,
            tables: args.table,
            databases: args.database,
            min_query_duration: args.min_query_duration,
            min_read_rows: args.min_read_rows,
            min_read_data: args.min_read_data,
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
    pub password: secrecy::SecretString,
    pub urls: Vec<String>,
    pub accept_invalid_certificate: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct PrintableContextProfile<'a> {
    pub user: &'a str,
    pub password: &'a str,
    pub urls: &'a Vec<String>,
    pub accept_invalid_certificate: bool,
}

impl ContextProfile {
    pub fn to_printable(&self, show_secrets: bool) -> PrintableContextProfile<'_> {
        let password = if show_secrets {
            // SAFETY: this is only done at the very edge of your app
            // and never stored back into your model.
            self.password.expose_secret()
        } else {
            "[REDACTED]"
        };
        PrintableContextProfile {
            user: &self.user,
            password,
            urls: &self.urls,
            accept_invalid_certificate: self.accept_invalid_certificate,
        }
    }
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
