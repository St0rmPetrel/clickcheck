use clickhouse::Row;
use serde::Deserialize;
use time::OffsetDateTime;

#[derive(Row, Deserialize, Debug)]
pub struct QueryLog<'a> {
    pub query: &'a str,
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

impl QueryLog<'_> {
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

#[derive(Debug)]
pub struct WeightedQueryLog<'a> {
    pub weight: u64,
    pub log: QueryLog<'a>,
}

impl<'a> PartialEq for WeightedQueryLog<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.weight == other.weight
    }
}

impl<'a> Eq for WeightedQueryLog<'a> {}

impl<'a> PartialOrd for WeightedQueryLog<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.weight.partial_cmp(&other.weight)
    }
}

impl<'a> Ord for WeightedQueryLog<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.weight.cmp(&other.weight)
    }
}
