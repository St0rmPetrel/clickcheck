#[tokio::main]
async fn main() {
    if let Err(err) = clickcheck::run().await {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}
