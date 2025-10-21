//! Program entry wiring together the async runtime and the application runner.
//! Exits with a non-zero code when the high-level workflow fails.
#[tokio::main]
async fn main() {
    if let Err(err) = ecs160_hw1::app::run().await {
        eprintln!("application error: {err}");
        std::process::exit(1);
    }
}
