use crate::model::{OutputFormat, SortBy};
use clap::{Parser, Subcommand};

/// Analyze ClickHouse query_log for inefficient queries.
#[derive(Parser)]
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

    #[clap(long, default_value = "text")]
    pub out: OutputFormat,

    /// Subcommands for different analysis modes
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Top analysis: queries, tables or users
    Top {
        #[command(subcommand)]
        command: TopCommand,

        /// number of output queries
        #[arg(long, default_value_t = 5)]
        limit: usize,

        /// Field to sort queries by in top results, descending order.
        #[clap(long, default_value = "weight")]
        sort_by: SortBy,
    },
}

#[derive(Subcommand)]
pub enum TopCommand {
    /// Show top N heavy queries
    Queries,
}
