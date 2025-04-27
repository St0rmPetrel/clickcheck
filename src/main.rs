mod analyzer;
mod cli;
mod client;
mod model;
mod output;

use clap::Parser;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let args = cli::CliArgs::parse();

    let client = client::Client::new(client::Config {
        urls: &args.urls,
        user: &args.user,
        password: &args.password,
    });

    let (tx, rx) = mpsc::channel(128);

    let analyzer_task = analyzer::top_queries_by_weight(rx, 5);
    let stream_task = client.stream_query_logs(tx);

    let (stream_result, top_queries) = tokio::join!(stream_task, analyzer_task);
    stream_result.expect("stream_query_logs failed");

    output::print_top(&top_queries, &args.out);
}
