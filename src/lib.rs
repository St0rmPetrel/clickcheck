#![doc = include_str!("../README.md")]
//! See [`command`] for available CLI commands.

pub mod analyzer;
pub mod cli;
pub mod client;
pub mod command;
pub mod context;
pub mod model;
pub mod output;

use clap::Parser;
use cli::{CliArgs, Command};

pub async fn run() -> Result<(), String> {
    let cli_args = CliArgs::parse();

    match &cli_args.command {
        Command::Queries {
            conn,
            sort_by,
            filter,
            limit,
        } => {
            let ctx = context::Context::new(cli_args.config.as_ref(), cli_args.context.as_deref())
                .map_err(|e| format!("context error: {e}"))?;
            let profile = resolve_profile(&conn, &ctx)?;
            let client = client::Client::new(client::Config {
                urls: &profile.urls,
                user: &profile.user,
                password: &profile.password,
                danger_accept_invalid_certs: profile.accept_invalid_certificate,
            })
            .map_err(|e| format!("create clickhouse client error: {e}"))?;
            command::top_queries(
                client,
                model::TopQueriesRequest {
                    sort_by: sort_by.clone(),
                    filter: filter.clone().into(),
                    limit: limit.clone(),
                    out: cli_args.out,
                },
            )
            .await?
        }
        Command::Total { conn, filter } => {
            let ctx = context::Context::new(cli_args.config.as_ref(), cli_args.context.as_deref())
                .map_err(|e| format!("context error: {e}"))?;
            let profile = resolve_profile(&conn, &ctx)?;
            let client = client::Client::new(client::Config {
                urls: &profile.urls,
                user: &profile.user,
                password: &profile.password,
                danger_accept_invalid_certs: profile.accept_invalid_certificate,
            })
            .map_err(|e| format!("create clickhouse client error: {e}"))?;
            command::total_queries(
                client,
                model::TotalQueriesRequest {
                    filter: filter.clone().into(),
                    out: cli_args.out,
                },
            )
            .await?
        }
        Command::Inspect {
            conn,
            fingerprint,
            filter,
        } => {
            let ctx = context::Context::new(cli_args.config.as_ref(), cli_args.context.as_deref())
                .map_err(|e| format!("context error: {e}"))?;
            let profile = resolve_profile(&conn, &ctx)?;
            let client = client::Client::new(client::Config {
                urls: &profile.urls,
                user: &profile.user,
                password: &profile.password,
                danger_accept_invalid_certs: profile.accept_invalid_certificate,
            })
            .map_err(|e| format!("create clickhouse client error: {e}"))?;
            command::inspect_fingerprint(
                client,
                model::InspectFingerprintRequest {
                    fingerprint: fingerprint.clone(),
                    filter: filter.clone().into(),
                    out: cli_args.out,
                },
            )
            .await?
        }
        Command::Errors {
            conn,
            filter,
            limit,
        } => {
            let ctx = context::Context::new(cli_args.config.as_ref(), cli_args.context.as_deref())
                .map_err(|e| format!("context error: {e}"))?;
            let profile = resolve_profile(&conn, &ctx)?;
            let client = client::Client::new(client::Config {
                urls: &profile.urls,
                user: &profile.user,
                password: &profile.password,
                danger_accept_invalid_certs: profile.accept_invalid_certificate,
            })
            .map_err(|e| format!("create clickhouse client error: {e}"))?;
            command::top_errors(
                client,
                model::TopErrorsRequest {
                    limit: limit.clone(),
                    filter: filter.clone().into(),
                    out: cli_args.out,
                },
            )
            .await?
        }
        Command::Context { command } => {
            let mut ctx =
                context::Context::new(cli_args.config.as_ref(), cli_args.context.as_deref())
                    .map_err(|e| format!("context error: {e}"))?;
            command::context(&mut ctx, command, cli_args.out).await?
        }
    }

    Ok(())
}

/// Centralized profile resolution:
/// 1. If `--context` or `current` is set, use that ContextProfile.
/// 2. Otherwise fall back to CLI flags (and error if missing).
fn resolve_profile(
    cli: &cli::ConnectArgs,
    ctx: &context::Context,
) -> Result<model::ContextProfile, String> {
    if let Some(profile) = ctx.profile().map_err(|e| format!("error: {e}"))? {
        let mut profile = profile.clone();
        if !cli.urls.is_empty() {
            profile.urls = cli.urls.clone();
        }
        if let Some(user) = cli.user.as_deref() {
            profile.user = user.to_string();
        }
        if let Some(password) = cli.password.clone() {
            profile.password = password;
        }
        if cli.interactive_password {
            let user = &profile.user;
            let password = rpassword::prompt_password(format!("ClickHouse {user} password: "))
                .map_err(|e| format!("read password from prompt: {e}"))?;
            profile.password = secrecy::SecretString::new(password.into());
        }
        if let Some(_) = cli.accept_invalid_certificate {
            profile.accept_invalid_certificate = true
        }
        return Ok(profile);
    };

    // no context/profile → require CLI flags
    if cli.urls.is_empty() {
        return Err("missing `--url`: supply at least one URL or set a context".into());
    }
    let user = cli
        .user
        .clone()
        .ok_or("missing `--user`: supply it or set a context")?;
    let password = cli
        .password
        .clone()
        .unwrap_or(secrecy::SecretString::new("".to_string().into()));
    let accept_invalid_certificate = if let Some(_) = cli.accept_invalid_certificate {
        true
    } else {
        false
    };

    Ok(model::ContextProfile {
        urls: cli.urls.clone(),
        user,
        password,
        accept_invalid_certificate,
    })
}
