// Copyright (c) 2025 Telman Rzaev
// Licensed under the MIT License (see LICENSE file for details)
#[tokio::main]
async fn main() {
    if let Err(err) = clickcheck::run().await {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}
