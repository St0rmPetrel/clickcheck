use crate::model::{OutputFormat, QueriesSortBy};
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
    /// Show top queries group by normalized_query_hash
    Queries {
        #[clap(flatten)]
        conn: ConnectArgs,

        /// Field to sort queries by in top results, descending order.
        #[arg(long, default_value = "weight")]
        sort_by: QueriesSortBy,

        #[clap(flatten)]
        filter: QueriesFilterArgs,

        /// number of output queries
        #[arg(long, default_value_t = 5)]
        limit: usize,
    },

    /// Show top errors
    Errors {
        #[clap(flatten)]
        conn: ConnectArgs,

        #[clap(flatten)]
        filter: ErrorFilterArgs,

        /// number of output queries
        #[arg(long, default_value_t = 5)]
        limit: usize,
    },

    /// Manage connection contexts (profiles) for ClickHouse clusters.
    Context {
        #[command(subcommand)]
        command: ContextCommand,
    },
}

#[derive(Args, Clone, Debug)]
pub struct ConnectArgs {
    /// ClickHouse node URL (can be specified multiple times)
    #[arg(short = 'U', long = "url")]
    pub urls: Vec<String>,

    /// ClickHouse username
    #[arg(short = 'u', long)]
    pub user: Option<String>,

    /// ClickHouse password
    #[arg(short = 'p', long)]
    pub password: Option<String>,

    /// Accept invalid (e.g., self-signed) TLS certificates when connecting over HTTPS.
    ///
    /// This option is useful when connecting to ClickHouse instances with self-signed
    /// or untrusted certificates. It **disables certificate validation**, which can be
    /// helpful for development or internal environments, but is **not recommended for production**
    /// due to potential security risks.
    #[arg(long, default_value = "false")]
    pub accept_invalid_certificate: bool,
}

#[derive(Args, Clone)]
pub struct QueriesFilterArgs {
    /// Lower bound for event_time (inclusive). Supports RFC3339 or YYYY-MM-DD.
    /// Examples: "2024-05-04T15:00:00Z", "2024-05-04"
    #[arg(long,value_parser = parse_datetime)]
    pub from: Option<OffsetDateTime>,
    /// Upper bound for event_time (exclusive). Supports RFC3339 or YYYY-MM-DD.
    /// Examples: "2024-05-04T15:00:00Z", "2024-05-04"
    #[arg(long,value_parser = parse_datetime)]
    pub to: Option<OffsetDateTime>,
}

#[derive(Args, Debug, Clone)]
pub struct ErrorFilterArgs {
    /// Only include errors that occurred within the last specified time period.
    /// Accepts human-readable durations like '15days 2min 2s', etc
    #[arg(long, value_parser = humantime::parse_duration)]
    pub last: Option<std::time::Duration>,
    /// Filter out errors that occurred fewer than N times across all nodes.
    /// Useful to focus on recurring or high-impact issues.
    #[arg(long)]
    pub min_count: Option<usize>,
    /// Filter errors by specific ClickHouse error code.
    /// Can be used multiple times to include multiple codes.
    #[arg(long)]
    pub code: Vec<i32>,
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

        /// Accept invalid (e.g., self-signed) TLS certificates when connecting over HTTPS.
        ///
        /// This option is useful when connecting to ClickHouse instances with self-signed
        /// or untrusted certificates. It **disables certificate validation**, which can be
        /// helpful for development or internal environments, but is **not recommended for production**
        /// due to potential security risks.
        #[arg(long, default_value = "false")]
        accept_invalid_certificate: bool,
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
