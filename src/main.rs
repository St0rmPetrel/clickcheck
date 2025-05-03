#[tokio::main]
async fn main() {
    if let Err(err) = ch_query_analyzer::run().await {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}
