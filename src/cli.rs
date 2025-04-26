use clap::Parser;

/// Analyze ClickHouse query_log for inefficient queries.
#[derive(Parser, Debug)]
#[command(
    name = "ch-query-analyzer",
    version,
    about = "Analyze ClickHouse query_log and group similar queries by fingerprint"
)]
pub struct CliArgs {
    /// ClickHouse server URL (e.g. http://localhost:8123)
    #[arg(long)]
    pub url: String,

    /// ClickHouse username
    #[arg(short = 'u', long, default_value = "default")]
    pub user: String,

    /// ClickHouse password
    #[arg(short = 'p', long, default_value = "")]
    pub password: String,
}
