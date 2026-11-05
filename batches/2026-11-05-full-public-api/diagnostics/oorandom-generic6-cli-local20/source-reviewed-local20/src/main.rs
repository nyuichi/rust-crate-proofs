mod aeneas;
mod ascii_vec;
mod api;
mod config;
mod external_blanket;
mod oorandom_generic;
mod git;
mod publish;
mod record;
mod scan;
mod state;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "cargo proofs",
    bin_name = "cargo proofs",
    version,
    about = "Publish Rust verification reports to proofs.rs"
)]
struct Cli {
    /// Service origin. Credentials and publication state are isolated per server.
    #[arg(
        long,
        global = true,
        env = "PROOFS_SERVER",
        default_value = "https://proofs.rs"
    )]
    server: String,
    #[command(subcommand)]
    command: Commands,
}
#[derive(Args, Clone, Default)]
pub struct ProjectArgs {
    #[arg(long, global = true)]
    manifest_path: Option<PathBuf>,
    /// Select a workspace package.
    #[arg(short = 'p', long, global = true)]
    package: Option<String>,
    #[arg(long, value_delimiter = ',', global = true)]
    features: Vec<String>,
    #[arg(long, global = true)]
    all_features: bool,
    #[arg(long, global = true)]
    no_default_features: bool,
    /// Target used for evaluating conditional compilation (does not run verification).
    #[arg(long, global = true)]
    target: Option<String>,
}
#[derive(Subcommand)]
enum Commands {
    /// Create proofs.toml next to the selected Cargo.toml.
    Init {
        #[command(flatten)]
        project: ProjectArgs,
        #[arg(long)]
        title: Option<String>,
        /// Verification tool used for the existing verification.
        #[arg(long, default_value = "kani", value_parser = ["kani", "creusot", "aeneas"])]
        tool: String,
        /// Creusot API selection (required for Creusot). Not the compilation --target.
        #[arg(long, value_enum, required_if_eq("tool", "creusot"))]
        tool_target: Option<config::CreusotTarget>,
        /// Tool version used for the existing verification (detected if omitted).
        #[arg(long)]
        tool_version: Option<String>,
    },
    /// Authorize this CLI in a browser.
    Login {
        #[arg(long)]
        no_browser: bool,
    },
    /// Revoke and remove this server's saved token.
    Logout,
    /// Verify a detached Git worktree and record results for publication.
    Run {
        #[command(flatten)]
        project: ProjectArgs,
        #[arg(last=true, required=true, num_args=2..)]
        command: Vec<String>,
    },
    /// Publish recorded SARIF results with embedded execution logs.
    Publish {
        #[command(flatten)]
        project: ProjectArgs,
        /// Preview recorded evidence without uploading; may prepare the API catalogue.
        #[arg(long)]
        dry_run: bool,
        /// Select a recorded run (defaults to the latest run).
        #[arg(long)]
        run: Option<String>,
        /// Attach to an existing report (e.g. from another machine).
        #[arg(long)]
        report: Option<u64>,
        /// Overwrite conflicting web edits after displaying the differences.
        #[arg(long)]
        force: bool,
        /// Explicitly approve removal of claims; does not bypass conflicts or recorded-run checks.
        #[arg(long)]
        yes: bool,
        /// Retry an interrupted publication using its exact saved request.
        #[arg(long, conflicts_with_all = ["dry_run", "run", "report", "force", "yes"])]
        resume: bool,
    },
}
fn run() -> Result<()> {
    let mut args: Vec<_> = std::env::args_os().collect();
    if args.get(1).is_some_and(|x| x == "proofs") {
        args.remove(1);
    }
    let cli = Cli::parse_from(args);
    let server = api::server(&cli.server)?;
    match cli.command {
        Commands::Init {
            project,
            title,
            tool_version,
            tool,
            tool_target,
        } => config::init(&project, title, tool_version, tool, tool_target),
        Commands::Login { no_browser } => api::login(&server, no_browser),
        Commands::Logout => api::logout(&server),
        Commands::Run { project, command } => record::run(&project, command),
        Commands::Publish {
            project,
            dry_run,
            run,
            report,
            force,
            yes,
            resume,
        } => publish::run(
            &server,
            &project,
            publish::Options {
                dry_run,
                run,
                report,
                force,
                yes,
                resume,
            },
        ),
    }
}
fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}
