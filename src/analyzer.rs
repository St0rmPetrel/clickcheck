use crate::model::{QueryLog, WeightedQueryLog};
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use tokio::sync::mpsc::Receiver;

pub struct Analyzer;

impl Analyzer {
    pub async fn find_top_heavy_queries(mut receiver: Receiver<QueryLog>) {
        const TOP_N: usize = 5;
        let mut heap = BinaryHeap::with_capacity(TOP_N + 1);
        let mut total_weight: u64 = 0;

        while let Some(row) = receiver.recv().await {
            let weighted = WeightedQueryLog {
                weight: row.weight(),
                log: row,
            };

            total_weight += weighted.weight;

            heap.push(Reverse(weighted));

            if heap.len() > TOP_N {
                heap.pop();
            }
        }

        let results = heap.into_sorted_vec();

        for (i, Reverse(weighted_log)) in results.iter().enumerate() {
            let percent = (weighted_log.weight as f64 / total_weight as f64) * 100.0;
            println!(
                "{}. weight: {}\nquery: {}\n",
                i + 1,
                percent,
                weighted_log.log.query,
            );
        }
    }
}
