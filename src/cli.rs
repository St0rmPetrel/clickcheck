use clap::Parser;

/// Analyze ClickHouse query_log for inefficient queries.
#[derive(Parser, Debug)]
#[command(
    name = "ch-query-analyzer",
    version,
    about = "Analyze ClickHouse query_log and group similar queries by fingerprint"
)]
pub struct CliArgs {
    /// ClickHouse node URL (can be specified multiple times)
    #[arg(short = 'U', long = "url", required = true)]
    pub urls: Vec<String>,

    /// ClickHouse username
    #[arg(short = 'u', long, default_value = "default")]
    pub user: String,

    /// ClickHouse password
    #[arg(short = 'p', long, default_value = "")]
    pub password: String,
}
