use crate::model::QueryLog;
use clickhouse::Client as ChClient;
use tokio::sync::mpsc::Sender;

pub struct Client(ChClient);

pub struct Config<'a> {
    pub url: &'a str,
    pub user: &'a str,
    pub password: &'a str,
}

impl Client {
    pub fn new(cfg: Config) -> Self {
        let inner = ChClient::default()
            .with_url(cfg.url)
            .with_user(cfg.user)
            .with_password(cfg.password)
            .with_database("system");

        Self(inner)
    }

    pub async fn stream_query_logs(&self, sender: Sender<QueryLog<'_>>) {
        let mut cursor = self
            .0
            .query(
                r#"
                SELECT
                    any(query) AS query,
                    max(event_time) AS max_event_time,
                    min(event_time) AS min_event_time,
                    sum(query_duration_ms) AS query_duration_ms,
                    sum(read_rows) AS read_rows,
                    sum(read_bytes) AS read_bytes,
                    sum(memory_usage) AS memory_usage,
                    sum(ProfileEvents['UserTimeMicroseconds']) AS user_time_us,
                    sum(ProfileEvents['SystemTimeMicroseconds']) AS system_time_us
                FROM query_log
                WHERE type != 'QueryStart'
                  AND query_kind = 'Select'
                GROUP BY normalized_query_hash
                "#,
            )
            .fetch::<QueryLog<'_>>()
            .unwrap();

        while let Some(row) = cursor.next().await.unwrap() {
            if sender.send(row).await.is_err() {
                break; // если получатель закрылся, выходим
            }
        }
    }
}
