use crate::model::{QueriesSortBy, QueryLog, WeightedQueryLog};
use std::collections::HashMap;
use tokio::sync::mpsc::Receiver;

struct Analyzer {
    queries: HashMap<u64, QueryLog>,
    total_weight: u64,
}

pub async fn top_queries(
    receiver: Receiver<QueryLog>,
    limit: usize,
    sort_by: QueriesSortBy,
) -> Vec<WeightedQueryLog> {
    let mut analyzer = Analyzer::new();

    analyzer.collect_logs(receiver).await;

    analyzer.top_queries(limit, sort_by)
}

impl Analyzer {
    // Create a new Analyzer
    fn new() -> Self {
        Self {
            queries: HashMap::new(),
            total_weight: 0,
        }
    }

    fn merge_query(&mut self, log: QueryLog) {
        self.total_weight += log.weight();

        self.queries
            .entry(log.normalized_query_hash)
            .and_modify(|existing| {
                existing.query_duration_ms += log.query_duration_ms;
                existing.read_rows += log.read_rows;
                existing.read_bytes += log.read_bytes;
                existing.memory_usage += log.memory_usage;
                existing.user_time_us += log.user_time_us;
                existing.system_time_us += log.system_time_us;

                if log.max_event_time > existing.max_event_time {
                    existing.max_event_time = log.max_event_time;
                }
                if log.min_event_time < existing.min_event_time {
                    existing.min_event_time = log.min_event_time;
                }
            })
            .or_insert(log);
    }

    async fn collect_logs(&mut self, mut rx: Receiver<QueryLog>) {
        while let Some(log) = rx.recv().await {
            self.merge_query(log);
        }
    }

    fn top_queries(&self, limit: usize, sort_by: QueriesSortBy) -> Vec<WeightedQueryLog> {
        let mut top_queries: Vec<_> = self.queries.values().collect();

        top_queries.sort_by_key(|q| {
            std::cmp::Reverse(match sort_by {
                QueriesSortBy::Weight => q.weight(),
                QueriesSortBy::CpuTime => q.system_time_us + q.system_time_us,
                QueriesSortBy::QueryDuration => q.query_duration_ms,
                QueriesSortBy::ReadRows => q.read_rows,
                QueriesSortBy::ReadBytes => q.read_bytes,
                QueriesSortBy::MemoryUsage => q.memory_usage,
                QueriesSortBy::UserTime => q.user_time_us,
                QueriesSortBy::SystemTime => q.system_time_us,
            })
        });
        top_queries.truncate(limit);

        top_queries
            .into_iter()
            .map(|q| WeightedQueryLog {
                cpu_time_us: q.system_time_us + q.user_time_us,
                weight: q.weight(),
                total_weight: self.total_weight,
                query: q.clone(),
            })
            .collect()
    }
}
