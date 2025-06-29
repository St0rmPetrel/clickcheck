//! Command-line interface definition for `clickcheck`.
//!
//! This module defines the CLI structure using [`clap`] derive macros. It handles:
//!
//! - Global arguments like config file path, output format, and active context.
//! - Subcommands:
//!   - `queries`: Analyze and group normalized ClickHouse queries with filtering.
//!   - `errors`: Display frequent ClickHouse query errors with filtering.
//!   - `context`: Manage named connection profiles (contexts).
//!
//! This module also includes utility functions to parse human-friendly inputs
//! like durations, dates, and secrets.
//!
//! The structure is designed to separate configuration parsing from execution logic,
//! making it easier to test and extend.
use crate::model::{OutputFormat, QueriesSortBy};
use clap::{ArgGroup, Args, Parser, Subcommand};
use std::path::PathBuf;
use std::str::FromStr;
use time::format_description::well_known::Rfc3339;
use time::macros::format_description;
use time::{Date, OffsetDateTime, Time};

/// Analyze ClickHouse query_log and system tables to detect inefficient queries,
/// anomalies, storage growth, and other potential issues for DBAs and SREs.
#[derive(Parser)]
#[command(
    name = "clickcheck",
    version,
    about = "Tool to analyze ClickHouse system tables, to detect potential issues for DBAs."
)]
pub struct CliArgs {
    /// The subcommand to execute (e.g. `queries`, `errors`, `context`).
    #[command(subcommand)]
    pub command: Command,

    /// Path to a context configuration TOML file.
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,

    /// Optional override for which context (profile) to use.
    /// Takes precedence over the stored default.
    #[arg(long, global = true)]
    pub context: Option<String>,

    /// Output format for results: text (default), json, or yaml.
    #[clap(long, global = true, default_value = "text")]
    pub out: OutputFormat,
}

/// Subcommands for different analysis modes.
#[derive(Subcommand)]
pub enum Command {
    /// Show top queries grouped by normalized_query_hash, with optional filters and sorting.
    Queries {
        #[clap(flatten)]
        conn: ConnectArgs,

        /// Field to sort queries by in top results, descending order.
        #[arg(long, default_value = "total-impact")]
        sort_by: QueriesSortBy,

        #[clap(flatten)]
        filter: QueriesFilterArgs,

        /// number of output queries
        #[arg(long, default_value_t = 5)]
        limit: usize,
    },

    /// Analyze total number of queries and aggregated statistics (e.g. read rows/data) in a time range.
    ///
    /// This command shows cumulative metrics over the specified filter window,
    /// helping track overall workload volume.
    Total {
        #[clap(flatten)]
        conn: ConnectArgs,

        #[clap(flatten)]
        filter: QueriesFilterArgs,
    },

    /// Show top ClickHouse query errors with filtering options.
    Errors {
        #[clap(flatten)]
        conn: ConnectArgs,

        #[clap(flatten)]
        filter: ErrorFilterArgs,

        /// number of output queries
        #[arg(long, default_value_t = 5)]
        limit: usize,
    },

    /// Manage context profiles used for connecting to ClickHouse.
    Context {
        #[command(subcommand)]
        command: ContextCommand,
    },
}

/// Connection-related arguments used in multiple commands.
#[derive(Args, Clone, Debug)]
pub struct ConnectArgs {
    /// ClickHouse node URL (can be specified multiple times)
    #[arg(short = 'U', long = "url")]
    pub urls: Vec<String>,

    /// ClickHouse username
    #[arg(short = 'u', long)]
    pub user: Option<String>,

    /// ClickHouse password
    #[arg(short = 'p', long, value_parser = parse_secret_arg)]
    pub password: Option<secrecy::SecretString>,

    /// ClickHouse password from interactive prompt
    #[arg(short = 'i', long, conflicts_with = "password")]
    pub interactive_password: bool,

    /// Accept invalid (e.g., self-signed) TLS certificates when connecting over HTTPS.
    ///
    /// This option is useful when connecting to ClickHouse instances with self-signed
    /// or untrusted certificates. It **disables certificate validation**, which can be
    /// helpful for development or internal environments, but is **not recommended for production**
    /// due to potential security risks.
    #[arg(long)]
    pub accept_invalid_certificate: Option<bool>,
}

/// Filters for narrowing down which queries to include in `queries` analysis.
/// Supports both absolute date ranges and relative durations.
#[derive(Args, Clone)]
#[command(group(
    ArgGroup::new("from_or_last")
        .args(["from", "last"])
        .required(true)
))]
pub struct QueriesFilterArgs {
    /// Lower bound for event_time (inclusive). Supports RFC3339 or YYYY-MM-DD.
    /// Examples: "2024-05-04T15:00:00Z", "2024-05-04"
    #[arg(
        long,
        value_parser = parse_datetime,
        group = "from_or_last"
    )]
    pub from: Option<OffsetDateTime>,
    /// Upper bound for event_time (exclusive). Supports RFC3339 or YYYY-MM-DD.
    /// Examples: "2024-05-04T15:00:00Z", "2024-05-04"
    #[arg(long, value_parser = parse_datetime)]
    pub to: Option<OffsetDateTime>,

    /// Only include queries from the last specified time period
    /// Accepts human-readable durations like '15days 2min 2s', etc
    #[arg(
        long,
        value_parser = humantime::parse_duration,
        group = "from_or_last"
    )]
    pub last: Option<std::time::Duration>,

    /// Filter by the user who executed the query. Can be specified multiple times.
    #[arg(long = "query-user")]
    pub query_user: Vec<String>,
    /// Filter by database name. Can be specified multiple times.
    #[arg(long)]
    pub database: Vec<String>,
    /// Filter by table name. Can be specified multiple times.
    #[arg(long)]
    pub table: Vec<String>,

    /// Filter by minimum query duration (e.g., 100ms, 1s)
    #[arg(long, value_parser = humantime::parse_duration)]
    pub min_query_duration: Option<std::time::Duration>,
    /// Filter by minimum number of rows read.
    #[arg(long)]
    pub min_read_rows: Option<u64>,
    /// Filter by the minimum amount of data read (supports units like B, KB, MB, GiB)
    #[arg(long, value_parser = bytesize::ByteSize::from_str)]
    pub min_read_data: Option<bytesize::ByteSize>,
}

/// Filters for the `errors` command.
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

/// Subcommands for inspecting or modifying context profiles.
#[derive(Subcommand)]
pub enum ContextCommand {
    /// Show config file which store context profiles
    ConfigPath,
    /// List all available context profiles
    List,
    /// Show the active context (CLI override or stored default)
    Current,
    /// Show details for a specific profile by name
    Show {
        name: String,
        /// Show sensitive information like passwords
        #[arg(long, default_value = "false")]
        show_secrets: bool,
    },
    /// Commands to add or modify context profiles
    Set {
        #[command(subcommand)]
        command: ContextSetCommand,
    },
    /// Commands to delete context profiles
    Delete { name: String },
}

/// Subcommands to set context values (profile definition or current profile).
#[derive(Subcommand)]
pub enum ContextSetCommand {
    /// Create or update a context profile
    Profile(SetProfileArgs),
    /// Set the stored default context to an existing profile
    Current { name: String },
}

/// Arguments for creating or updating a context profile.
/// Requires either a password or interactive prompt (enforced by ArgGroup).
#[derive(Args)]
#[command(group( ArgGroup::new("auth") .args(["password", "interactive_password"]) .required(true)))]
pub struct SetProfileArgs {
    /// The name of the profile to create or update
    pub name: String,

    /// ClickHouse node URLs
    #[arg(short = 'U', long = "url", required = true)]
    pub urls: Vec<String>,

    /// ClickHouse username
    #[arg(short = 'u', long, required = true)]
    pub user: String,

    /// ClickHouse password (plaintext)
    #[arg( short = 'p', long, value_parser = parse_secret_arg, group = "auth")]
    pub password: Option<secrecy::SecretString>,

    /// Get password via interactive prompt
    #[arg(short = 'i', long, group = "auth")]
    pub interactive_password: bool,

    /// Accept invalid (e.g., self-signed) TLS certificates when connecting over HTTPS.
    ///
    /// This option is useful when connecting to ClickHouse instances with self-signed
    /// or untrusted certificates. It **disables certificate validation**, which can be
    /// helpful for development or internal environments, but is **not recommended for production**
    /// due to potential security risks.
    #[arg(long, default_value_t = false)]
    pub accept_invalid_certificate: bool,
}

/// Parses either a full RFC3339 timestamp or a YYYY-MM-DD date.
/// Returns an `OffsetDateTime` set to midnight if only a date is provided.
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

/// Parses a password from a CLI argument into a `SecretString`.
/// Used to avoid leaking secrets in logs or stack traces.
fn parse_secret_arg(s: &str) -> Result<secrecy::SecretString, String> {
    Ok(secrecy::SecretString::new(s.to_string().into()))
}
