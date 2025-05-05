use clap::ValueEnum;
use clickhouse::Row;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

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
pub enum SortBy {
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
pub struct Filter {
    pub from: Option<OffsetDateTime>,
    pub to: Option<OffsetDateTime>,
}

#[derive(Debug)]
pub struct TopQueryRequest {
    pub limit: usize,
    pub sort_by: SortBy,
    pub filter: Filter,
    pub out: OutputFormat,
}

impl From<cli::FilterArgs> for Filter {
    fn from(args: cli::FilterArgs) -> Self {
        Self {
            from: args.from,
            to: args.to,
        }
    }
}
