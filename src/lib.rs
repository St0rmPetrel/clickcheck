pub mod analyzer;
pub mod cli;
pub mod client;
pub mod command;
pub mod model;
pub mod output;

use clap::Parser;
use cli::{CliArgs, Command, TopCommand};

pub async fn run() -> Result<(), String> {
    let args = CliArgs::parse();

    let client = client::Client::new(client::Config {
        urls: &args.urls,
        user: &args.user,
        password: &args.password,
    });

    match args.command {
        Command::Top {
            command,
            limit,
            sort_by,
        } => match command {
            TopCommand::Queries => {
                command::handle_top_queries(
                    client,
                    model::TopQueryRequest {
                        limit,
                        sort_by,
                        out: args.out,
                    },
                )
                .await?
            }
        },
    }

    Ok(())
}
