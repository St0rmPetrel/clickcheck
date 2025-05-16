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

    let mut ctx = context::Context::new(cli_args.config.as_ref(), cli_args.context.as_deref())
        .map_err(|e| format!("context error: {e}"))?;

    match &cli_args.command {
        Command::Queries {
            limit,
            sort_by,
            filter,
            conn,
        } => {
            let profile = resolve_profile(&conn, &ctx)?;
            let client = client::Client::new(client::Config {
                urls: &profile.urls,
                user: &profile.user,
                password: &profile.password,
            });
            command::handle_top_queries(
                client,
                model::TopQueriesRequest {
                    limit: limit.clone(),
                    sort_by: sort_by.clone(),
                    filter: filter.clone().into(),
                    out: cli_args.out,
                },
            )
            .await?
        }
        Command::Context { command } => {
            command::handle_context(&mut ctx, command, cli_args.out).await?
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
    if let Some(profile) = ctx.profile() {
        let mut profile = profile.clone();
        if !cli.urls.is_empty() {
            profile.urls = cli.urls.clone();
        }
        if let Some(user) = cli.user.as_deref() {
            profile.user = user.to_string();
        }
        if let Some(password) = cli.password.as_deref() {
            profile.password = password.to_string();
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
    let password = cli.password.clone().unwrap_or("".to_string());

    Ok(model::ContextProfile {
        urls: cli.urls.clone(),
        user,
        password,
    })
}
