use crate::model::{Error, QueriesSortBy, QueryLog};
use std::collections::HashMap;
use tokio::sync::mpsc::Receiver;

struct Analyzer {
    queries: HashMap<u64, QueryLog>,
    errors: HashMap<i32, Error>,
}

pub async fn top_queries(
    receiver: Receiver<QueryLog>,
    limit: usize,
    sort_by: QueriesSortBy,
) -> Vec<QueryLog> {
    let mut analyzer = Analyzer::new();

    analyzer.collect_logs(receiver).await;

    analyzer.top_queries(limit, sort_by)
}

pub async fn top_errors(receiver: Receiver<Error>, limit: usize) -> Vec<Error> {
    let mut analyzer = Analyzer::new();

    analyzer.collect_errors(receiver).await;

    analyzer.top_errors(limit)
}

impl Analyzer {
    // Create a new Analyzer
    fn new() -> Self {
        Self {
            queries: HashMap::new(),
            errors: HashMap::new(),
        }
    }

    fn merge_query(&mut self, log: QueryLog) {
        self.queries
            .entry(log.normalized_query_hash)
            .and_modify(|existing| {
                // Базовые метрики (raw values)
                existing.total_query_duration_ms += log.total_query_duration_ms;
                existing.total_read_rows += log.total_read_rows;
                existing.total_read_bytes += log.total_read_bytes;
                existing.total_memory_usage += log.total_memory_usage;
                existing.total_user_time_us += log.total_user_time_us;
                existing.total_system_time_us += log.total_system_time_us;

                // Time bounds
                existing.max_event_time = existing.max_event_time.max(log.max_event_time);
                existing.min_event_time = existing.min_event_time.min(log.min_event_time);

                existing.users.extend(log.users.iter().cloned());
                existing.users.sort_unstable();
                existing.users.dedup();

                existing.databases.extend(log.databases.iter().cloned());
                existing.databases.sort_unstable();
                existing.databases.dedup();

                existing.tables.extend(log.tables.iter().cloned());
                existing.tables.sort_unstable();
                existing.tables.dedup();

                // Композитные показатели
                existing.io_impact += log.io_impact;
                existing.cpu_impact += log.cpu_impact;
                existing.memory_impact += log.memory_impact;
                existing.time_impact += log.time_impact;
                existing.total_impact += log.total_impact;
            })
            .or_insert(log);
    }

    fn merge_error(&mut self, err: Error) {
        self.errors
            .entry(err.code)
            .and_modify(|existing| {
                existing.count += err.count;
                if err.last_error_time > existing.last_error_time {
                    existing.last_error_time = err.last_error_time;
                }
            })
            .or_insert(err);
    }

    async fn collect_logs(&mut self, mut rx: Receiver<QueryLog>) {
        while let Some(log) = rx.recv().await {
            self.merge_query(log);
        }
    }

    async fn collect_errors(&mut self, mut rx: Receiver<Error>) {
        while let Some(err) = rx.recv().await {
            self.merge_error(err);
        }
    }

    fn top_queries(&self, limit: usize, sort_by: QueriesSortBy) -> Vec<QueryLog> {
        let mut top_queries: Vec<_> = self.queries.clone().into_values().collect();

        top_queries.sort_by_key(|q| {
            std::cmp::Reverse(match sort_by {
                QueriesSortBy::TotalImpact => q.total_impact,
                QueriesSortBy::IOImpact => q.io_impact,
                QueriesSortBy::CPUImpact => q.cpu_impact,
                QueriesSortBy::MemoryImpact => q.memory_impact,
                QueriesSortBy::TimeImpact => q.time_impact,
                QueriesSortBy::CpuTime => q.total_system_time_us + q.total_system_time_us,
                QueriesSortBy::QueryDuration => q.total_query_duration_ms,
                QueriesSortBy::ReadRows => q.total_read_rows,
                QueriesSortBy::ReadBytes => q.total_read_bytes,
                QueriesSortBy::MemoryUsage => q.total_memory_usage,
                QueriesSortBy::UserTime => q.total_user_time_us,
                QueriesSortBy::SystemTime => q.total_system_time_us,
            })
        });
        top_queries.truncate(limit);

        top_queries
    }

    fn top_errors(self, limit: usize) -> Vec<Error> {
        let mut top_errors: Vec<Error> = self.errors.values().cloned().collect();

        top_errors.sort_by_key(|e| (std::cmp::Reverse(e.count), e.code));
        top_errors.truncate(limit);

        top_errors
    }
}
