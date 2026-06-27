use clap::{Parser, Subcommand, ValueEnum};

use agent_body_core::cli::apply_progress_env;
use agent_body_core::ui::ProgressMode;

#[derive(Clone, Copy, Debug, ValueEnum)]
enum ProgressArg {
    Auto,
    Plain,
    Quiet,
}

impl From<ProgressArg> for ProgressMode {
    fn from(value: ProgressArg) -> Self {
        match value {
            ProgressArg::Auto => ProgressMode::Auto,
            ProgressArg::Plain => ProgressMode::Plain,
            ProgressArg::Quiet => ProgressMode::Quiet,
        }
    }
}

#[derive(Parser)]
#[command(version)]
#[command(name = "agent-eyes", about = "Observability and visual QA")]
struct Cli {
    /// Progress output style: auto, plain, or quiet
    #[arg(long, value_enum, global = true, default_value = "auto")]
    progress: ProgressArg,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start telemetry daemon (HTTP + MCP server)
    Serve,
    /// Start the MCP stdio server only (no HTTP daemon)
    ServeMcp,
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
    /// DOM indexing and search (SQLite at ~/.autonomic/memory/eyes_dom.db)
    Dom {
        #[command(subcommand)]
        command: DomCommands,
    },
    /// Native local vision (LLaVA via candle, requires --features vlm)
    Vlm {
        #[command(subcommand)]
        command: VlmCommands,
    },
    /// Update agent-eyes to the latest release
    Update {
        /// Force update even if already at latest version
        #[arg(long)]
        force: bool,
    },
    /// Show configuration and status
    Status,
    /// View supervisor logs
    Log {
        /// List available logs
        #[arg(long)]
        list: bool,
        /// Log name to print
        name: Option<String>,
        /// Follow (tail -f) the log
        #[arg(long)]
        follow: bool,
    },
}

#[derive(Subcommand)]
enum DomCommands {
    /// Fetch and index a URL into the DOM database
    Index {
        url: String,
        #[arg(long, default_value_t = 5000)]
        max_elements: usize,
    },
    /// Index a local HTML file
    File {
        path: std::path::PathBuf,
        #[arg(long, default_value_t = 5000)]
        max_elements: usize,
    },
    /// Show DOM index statistics
    Stats,
    /// Search indexed DOM elements
    Search {
        query: String,
        #[arg(long, default_value_t = 20)]
        limit: u32,
    },
}

#[derive(Subcommand)]
enum VlmCommands {
    /// Describe an image with the local LLaVA model
    Describe {
        image: std::path::PathBuf,
        #[arg(short, long)]
        prompt: Option<String>,
    },
    /// Show VLM configuration and compile status
    Status,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    apply_progress_env(cli.progress.into());
    match cli.command {
        Commands::Serve => {
            let config = agent_eyes::config::Config::load()?;
            agent_eyes::serve::start(config).await?;
        }
        Commands::ServeMcp => {
            let config = agent_eyes::config::Config::load()?;
            agent_eyes::mcp_server::EyesMcp::run(config).await?;
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
        Commands::Dom { command } => match command {
            DomCommands::Index { url, max_elements } => {
                let report = agent_eyes::dom_index::index_url(&url, max_elements).await?;
                println!("{}", serde_json::to_string_pretty(&report)?);
            }
            DomCommands::File { path, max_elements } => {
                let report = agent_eyes::dom_index::index_file(&path, max_elements)?;
                println!("{}", serde_json::to_string_pretty(&report)?);
            }
            DomCommands::Stats => {
                let stats = agent_eyes::dom_index::load_stats()?;
                println!("{}", serde_json::to_string_pretty(&stats)?);
            }
            DomCommands::Search { query, limit } => {
                let hits = agent_eyes::dom_index::search(&query, limit)?;
                println!("{}", serde_json::to_string_pretty(&hits)?);
            }
        },
        Commands::Vlm { command } => {
            let config = agent_eyes::config::Config::load()?;
            match command {
                VlmCommands::Describe { image, prompt } => {
                    let result =
                        agent_eyes::vlm::describe_image(&image, prompt.as_deref(), &config.vlm)
                            .await?;
                    println!("{}", serde_json::to_string_pretty(&result)?);
                }
                VlmCommands::Status => {
                    let status = agent_eyes::vlm::vlm_status(&config.vlm);
                    println!("{}", serde_json::to_string_pretty(&status)?);
                }
            }
        }
        Commands::Update { force } => {
            agent_eyes::update::run_update(force)?;
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
            println!("  dom db: {}", agent_eyes::dom_index::db_path().display());
            if let Ok(stats) = agent_eyes::dom_index::load_stats() {
                println!("  dom pages: {}", stats.pages);
                println!("  dom elements: {}", stats.elements);
            }
            let vlm = agent_eyes::vlm::vlm_status(&config.vlm);
            println!(
                "  vlm: enabled={} compiled={} model={}",
                vlm.enabled, vlm.feature_compiled, vlm.model_id
            );
        }
        Commands::Log { list, name, follow } => {
            if list {
                for log in agent_eyes::log::list_logs()? {
                    println!("{log}");
                }
            } else if let Some(name) = name {
                if follow {
                    agent_eyes::log::follow_log(&name)?;
                } else {
                    agent_eyes::log::print_log(&name)?;
                }
            }
        }
    }
    Ok(())
}
