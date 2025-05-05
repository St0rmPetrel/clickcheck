pub mod analyzer;
pub mod cli;
pub mod client;
pub mod command;
pub mod model;
pub mod output;

use clap::Parser;
use cli::{CliArgs, Command, TopCommand};

pub async fn run() -> Result<(), String> {
    let cli_args = CliArgs::parse();

    let client = client::Client::new(client::Config {
        urls: &cli_args.urls,
        user: &cli_args.user,
        password: &cli_args.password,
    });

    match cli_args.command {
        Command::Top { command, args } => match command {
            TopCommand::Queries => {
                command::handle_top_queries(
                    client,
                    model::TopQueryRequest {
                        limit: args.limit,
                        sort_by: args.sort_by,
                        filter: args.filter.into(),
                        out: cli_args.out,
                    },
                )
                .await?
            }
        },
    }

    Ok(())
}
