use ecs160_hw1::config::AppConfig;
use ecs160_hw1::error::AppError;
use ecs160_hw1::github::GitHubClient;

const TARGET_LANGUAGES: &[&str] = &["Java", "C", "C++", "Rust"];
const TOP_N: u8 = 10;

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("application error: {err}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), AppError> {
    let config = AppConfig::load()?;
    let client = GitHubClient::new(config.github.clone())?;

    for language in TARGET_LANGUAGES {
        match client.fetch_top_repositories(language, TOP_N).await {
            Ok(repos) => {
                println!(
                    "language `{language}`: retrieved {} repositories (stub output)",
                    repos.len()
                );
            }
            Err(AppError::NotImplemented) => {
                println!("`fetch_top_repositories` is not implemented yet; skipping `{language}`.");
            }
            Err(err) => {
                eprintln!("failed to fetch repos for `{language}`: {err}");
            }
        }
    }

    Ok(())
}
