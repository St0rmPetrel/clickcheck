//! # Command Handlers
//!
//! This module contains top-level async functions that handle the main subcommands of the CLI tool:
//!
//! - `queries`: Analyzes heavy queries grouped by `normalized_query_hash`
//! - `errors`: Displays top system errors grouped by error code
//! - `context`: Manages ClickHouse connection profiles
//!
//! These functions coordinate between [`cli`] (input CLI arguments), internal logic, and [`output`] (printing).
//!
//! ## Architecture
//!
//! Each handler typically:
//! 1. Starts a streaming task using the [`client`] module to read from ClickHouse system tables
//! 2. Spawns an analyzer task from the [`analyzer`] module to consume and process the stream
//! 3. Waits for both tasks concurrently via `tokio::join!`
//! 4. Renders the result via the [`output`] module

use crate::analyzer;
use crate::cli;
use crate::client;
use crate::context;
use crate::model;
use crate::output;
use tokio::sync::mpsc;

/// Executes the `queries` command by analyzing heavy queries in `system.query_log`.
///
/// Streams log entries grouped by `normalized_query_hash` and prints top queries
/// sorted by the selected impact metric.
pub async fn top_queries(
    client: client::Client,
    req: model::TopQueriesRequest,
) -> Result<(), String> {
    let (tx, rx) = mpsc::channel(128);
    let analyzer_task = analyzer::top_queries(rx, req.limit, req.sort_by);

    let stream_task = client.stream_logs_by_fingerprint(req.filter.into(), tx);

    let (stream_result, top_queries) = tokio::join!(stream_task, analyzer_task);

    stream_result.map_err(|e| format!("Stream error: {e}"))?;

    output::print_top_queries(&top_queries, req.out);

    Ok(())
}

/// Executes the `total-queries` command by aggregating total metrics from `system.query_log`.
///
/// Streams pre-aggregated [`QueryLogTotal`] entries and summarizes overall resource usage,
/// such as total I/O, CPU, memory, network, and time impact across all queries.
///
/// # Workflow
///
/// - Starts a streaming task from the [`client`] to receive [`model::QueryLogTotal`] records.
/// - Analyzes the incoming stream using [`analyzer::total_queries`].
/// - Outputs the aggregated totals using [`output::print_total_queries`].
///
/// # Arguments
///
/// - `client`: A configured ClickHouse client used to stream log data.
/// - `req`: User request specifying filter criteria and output format.
///
/// # Returns
///
/// `Result<(), String>` indicating success or a streaming error.
pub async fn total_queries(
    client: client::Client,
    req: model::TotalQueriesRequest,
) -> Result<(), String> {
    let (tx, rx) = mpsc::channel(128);
    let analyzer_task = analyzer::total_queries(rx);

    let stream_task = client.stream_logs_total(req.filter.into(), tx);

    let (stream_result, total_queries) = tokio::join!(stream_task, analyzer_task);

    stream_result.map_err(|e| format!("Stream error: {e}"))?;

    output::print_total_queries(&total_queries, req.out);

    Ok(())
}

/// Executes the `errors` command by analyzing top errors in `system.errors`.
///
/// Streams error entries grouped by error code and prints top recurring errors.
pub async fn top_errors(
    client: client::Client,
    req: model::TopErrorsRequest,
) -> Result<(), String> {
    let (tx, rx) = mpsc::channel(128);
    let analyzer_task = analyzer::top_errors(rx, req.limit);

    let stream_task = client.stream_error_by_code(req.filter.into(), tx);

    let (stream_result, top_errors) = tokio::join!(stream_task, analyzer_task);

    stream_result.map_err(|e| format!("Stream error: {e}"))?;

    output::print_top_errors(&top_errors, req.out);

    Ok(())
}

/// Handles the `context` CLI command.
///
/// This command is a wrapper around the [`mod@context`] module, providing access to
/// ClickHouse profile management via subcommands like `list`, `current`, `show`, and `set`.
///
/// It handles reading, modifying, and securely storing connection profiles defined
/// in a TOML configuration file.
pub async fn context(
    ctx: &mut context::Context,
    command: &cli::ContextCommand,
    out: model::OutputFormat,
) -> Result<(), String> {
    match command {
        cli::ContextCommand::ConfigPath => {
            let path = ctx.get_config_path();
            output::print_context_config_path(path, out);
        }
        cli::ContextCommand::List => {
            let names = ctx.list();
            output::print_context_list(&names, out);
        }

        cli::ContextCommand::Current => {
            let active = ctx.active_profile_name();
            output::print_context_current(active, out);
        }

        cli::ContextCommand::Show { name, show_secrets } => {
            let profile = ctx
                .get_profile(name)
                .map_err(|e| format!("show profile error: {}", e))?;

            let printable = profile.to_printable(*show_secrets);
            output::print_context_profile(&printable, out);
        }

        cli::ContextCommand::Delete { name } => ctx
            .delete_profile(name)
            .map_err(|e| format!("delete profile error: {e}"))?,

        cli::ContextCommand::Set { command } => match command {
            cli::ContextSetCommand::Current { name } => {
                ctx.set_default(name)
                    .map_err(|e| format!("set current error: {}", e))?;
            }
            cli::ContextSetCommand::Profile(args) => {
                let user = args.user.clone();
                let password = if args.interactive_password {
                    let password =
                        rpassword::prompt_password(format!("ClickHouse {user} password: "))
                            .map_err(|e| format!("read password from prompt: {e}"))?;
                    secrecy::SecretString::new(password.into())
                } else {
                    args.password.clone().unwrap()
                };
                ctx.set_profile(
                    model::ContextProfile {
                        user,
                        password,
                        urls: args.urls.clone(),
                        accept_invalid_certificate: args.accept_invalid_certificate.clone(),
                    },
                    &args.name,
                )
                .map_err(|e| format!("set profile error: {}", e))?;
            }
        },
    }

    Ok(())
}
