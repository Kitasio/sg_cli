use std::process;

#[tokio::main]
async fn main() {
    if let Err(e) = sg_cli::run().await {
        eprintln!("Error running the cli: {}", e);
        process::exit(1);
    }
}

