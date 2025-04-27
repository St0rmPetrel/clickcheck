mod analyzer;
mod cli;
mod client;
mod model;

use analyzer::Analyzer;
use clap::Parser;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let args = cli::CliArgs::parse();

    let client = client::Client::new(client::Config {
        url: &args.url,
        user: &args.user,
        password: &args.password,
    });

    let (tx, rx) = mpsc::channel(128);

    let analyzer_task = Analyzer::find_top_heavy_queries(rx);
    let stream_task = client.stream_query_logs(tx);

    let (stream_result, _) = tokio::join!(stream_task, analyzer_task);
    stream_result.expect("stream_query_logs failed");
}
