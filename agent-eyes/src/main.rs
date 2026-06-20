use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "agent-eyes", about = "Observability and visual QA")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start telemetry daemon
    Serve,
    /// Capture a screenshot of a URL
    Capture {
        /// URL to capture
        url: String,
        /// Output path (default: screenshot.png)
        #[arg(short, long, default_value = "screenshot.png")]
        output: std::path::PathBuf,
    },
    /// Compare two images (pixel diff)
    Diff {
        /// Reference image
        reference: std::path::PathBuf,
        /// Comparison image
        comparison: std::path::PathBuf,
        /// Output diff image
        #[arg(short, long, default_value = "diff.png")]
        output: std::path::PathBuf,
    },
    /// Show configuration and status
    Status,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    match cli.command {
        Commands::Serve => {
            println!("agent-eyes serve (not yet implemented)");
        }
        Commands::Capture { url, output } => {
            agent_eyes::capture::capture_url(&url, &output).await?;
        }
        Commands::Diff { reference, comparison, output } => {
            agent_eyes::diff::pixel_diff(&reference, &comparison, &output)?;
        }
        Commands::Status => {
            let _ = agent_eyes::config::Config::load()?;
            println!("agent-eyes status");
            println!("  config: {}", agent_eyes::config::Config::config_path().display());
        }
    }
    Ok(())
}
