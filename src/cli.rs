use crate::model::{OutputFormat, SortBy};
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;
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
    /// Subcommands for different analysis modes
    #[command(subcommand)]
    pub command: Command,

    /// Path to context config TOML file
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,

    /// Optional override for which context (profile) to use.
    /// If provided, this context name takes precedence over the stored default.
    #[arg(long, global = true)]
    pub context: Option<String>,

    #[clap(long, global = true, default_value = "text")]
    pub out: OutputFormat,
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

    /// Manage connection contexts (profiles) for ClickHouse clusters.
    Context {
        #[command(subcommand)]
        command: ContextCommand,
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
    #[arg(long, default_value_t = 5, global = true)]
    pub limit: usize,

    /// Field to sort queries by in top results, descending order.
    #[arg(long, default_value = "weight", global = true)]
    pub sort_by: SortBy,

    #[clap(flatten)]
    pub filter: FilterArgs,

    /// ClickHouse node URL (can be specified multiple times)
    #[arg(short = 'U', long = "url", global = true)]
    pub urls: Vec<String>,

    /// ClickHouse username
    #[arg(short = 'u', long, global = true)]
    pub user: Option<String>,

    /// ClickHouse password
    #[arg(short = 'p', long, global = true)]
    pub password: Option<String>,
}

#[derive(Args, Clone)]
pub struct FilterArgs {
    /// Lower bound for event_time (inclusive). Supports RFC3339 or YYYY-MM-DD.
    /// Examples: "2024-05-04T15:00:00Z", "2024-05-04"
    #[arg(long,value_parser = parse_datetime, global = true)]
    pub from: Option<OffsetDateTime>,
    /// Upper bound for event_time (exclusive). Supports RFC3339 or YYYY-MM-DD.
    /// Examples: "2024-05-04T15:00:00Z", "2024-05-04"
    #[arg(long,value_parser = parse_datetime, global = true)]
    pub to: Option<OffsetDateTime>,
}

#[derive(Subcommand)]
pub enum ContextCommand {
    /// List all available context profiles
    List,
    /// Show the active context (CLI override or stored default)
    Current,
    /// Show details for a specific profile by name
    Show { name: String },
    /// Commands to modify context profiles
    Set {
        #[command(subcommand)]
        command: ContextSetCommand,
    },
}

#[derive(Subcommand)]
pub enum ContextSetCommand {
    /// Create or update a context profile
    Profile {
        /// The name of the profile to create or update
        name: String,

        /// ClickHouse node URLs
        #[arg(short = 'U', long = "url", required = true)]
        urls: Vec<String>,

        /// ClickHouse username
        #[arg(short = 'u', long, required = true)]
        user: String,

        /// ClickHouse password
        #[arg(short = 'p', long, default_value = "")]
        password: String,
    },
    /// Set the stored default context to an existing profile
    Current { name: String },
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
