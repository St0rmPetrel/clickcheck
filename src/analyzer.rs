use crate::model::QueryLog;
use std::collections::HashMap;
use tokio::sync::mpsc::Receiver;

pub struct Analyzer {
    queries: HashMap<u64, QueryLog>,
    total_weight: u64,
}

impl Analyzer {
    // Create a new Analyzer
    pub fn new() -> Self {
        Self {
            queries: HashMap::new(),
            total_weight: 0,
        }
    }

    pub async fn find_top_heavy_queries(receiver: Receiver<QueryLog>) {
        const TOP_N: usize = 5;

        let mut analyzer = Analyzer::new();

        analyzer.collect_logs(receiver).await;

        analyzer.print_heavy_top_n(TOP_N);
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

    fn print_heavy_top_n(&self, n: usize) {
        let mut top_queries: Vec<_> = self.queries.values().collect();

        top_queries.sort_by_key(|q| std::cmp::Reverse(q.weight()));

        // Display the top 5 queries
        for (i, q) in top_queries.iter().take(n).enumerate() {
            let weight = q.weight();
            let percent = (weight as f64 / self.total_weight as f64) * 100.0;
            println!(
                "{}. weight: {}\nweight_percent: {}\nquery: {}\n",
                i + 1,
                weight,
                percent,
                q.query,
            );
        }
    }
}
