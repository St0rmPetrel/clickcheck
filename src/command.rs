use crate::analyzer;
use crate::cli;
use crate::client;
use crate::context;
use crate::model;
use crate::output;
use tokio::sync::mpsc;

pub async fn handle_top_queries(
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

pub async fn handle_top_errors(
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

pub async fn handle_context(
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
