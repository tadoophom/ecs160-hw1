//! Main entry point.
#[tokio::main]
async fn main() {
    if let Err(err) = ecs160_hw1::app::run().await {
        eprintln!("application error: {err}");
        std::process::exit(1);
    }
}
