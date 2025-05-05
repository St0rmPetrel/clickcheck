use crate::model::{OutputFormat, SortBy};
use clap::{Args, Parser, Subcommand};
use time::format_description::well_known::Rfc3339;
use time::macros::format_description;
use time::{Date, OffsetDateTime, Time};

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

        #[clap(flatten)]
        args: TopArgs,
    },
}

#[derive(Subcommand)]
pub enum TopCommand {
    /// Show top N heavy queries
    Queries,
}

#[derive(Args)]
pub struct TopArgs {
    /// number of output queries
    #[arg(long, default_value_t = 5)]
    pub limit: usize,

    /// Field to sort queries by in top results, descending order.
    #[arg(long, default_value = "weight")]
    pub sort_by: SortBy,

    #[clap(flatten)]
    pub filter: FilterArgs,
}

#[derive(Args)]
pub struct FilterArgs {
    /// Lower bound for event_time (inclusive). Supports RFC3339 or YYYY-MM-DD.
    /// Examples: "2024-05-04T15:00:00Z", "2024-05-04"
    #[arg(long,value_parser = parse_datetime)]
    pub from: Option<OffsetDateTime>,
    /// Upper bound for event_time (exclusive). Supports RFC3339 or YYYY-MM-DD.
    /// Examples: "2024-05-04T15:00:00Z", "2024-05-04"
    #[arg(long,value_parser = parse_datetime)]
    pub to: Option<OffsetDateTime>,
}

fn parse_datetime(s: &str) -> Result<OffsetDateTime, String> {
    if let Ok(dt) = OffsetDateTime::parse(s, &Rfc3339) {
        return Ok(dt);
    }

    let date_format = format_description!("[year]-[month]-[day]");
    if let Ok(date) = Date::parse(s, &date_format) {
        let date = date.with_time(Time::MIDNIGHT).assume_utc();
        return Ok(date);
    }

    Err("Invalid datetime format. Use RFC3339 (e.g. 2024-05-01T10:30:00Z) or YYYY-MM-DD.".into())
}
