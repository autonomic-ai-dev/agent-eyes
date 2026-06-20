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
    /// Describe a URL or file (extract structure, count elements)
    Describe {
        /// URL (e.g. localhost:3000) or file path
        target: String,
    },
    /// Verify UI against stored baseline before dataset generation
    Verify {
        /// URL (e.g. localhost:3000)
        target: String,
        /// Optional baseline image path
        #[arg(long)]
        baseline: Option<std::path::PathBuf>,
        /// Max allowed pixel diff percent (default 1.0)
        #[arg(long, default_value_t = 1.0)]
        threshold: f64,
        /// Update baseline from current capture
        #[arg(long)]
        update_baseline: bool,
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
            let config = agent_eyes::config::Config::load()?;
            agent_eyes::serve::start(config).await?;
        }
        Commands::Capture { url, output } => {
            agent_eyes::capture::capture_url(&url, &output).await?;
        }
        Commands::Diff {
            reference,
            comparison,
            output,
        } => {
            agent_eyes::diff::pixel_diff(&reference, &comparison, &output)?;
        }
        Commands::Describe { target } => {
            agent_eyes::describe::describe_target(&target).await?;
        }
        Commands::Verify {
            target,
            baseline,
            threshold,
            update_baseline,
        } => {
            let report = agent_eyes::verify::verify_ui(
                &target,
                baseline.as_deref(),
                threshold,
                update_baseline,
            )
            .await?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            if !report.passed {
                std::process::exit(1);
            }
        }
        Commands::Status => {
            let config = agent_eyes::config::Config::load()?;
            println!("agent-eyes status");
            println!(
                "  config: {}",
                agent_eyes::config::Config::config_path().display()
            );
            println!("  server port: {}", config.server.port);
            println!("  spine url: {}", config.spine.url);
        }
    }
    Ok(())
}
