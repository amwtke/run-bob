//! run-bob: A CLI to bootstrap Bob's 4-ring Clean Architecture + Superpowers harness for Claude Code and Codex projects.

use anyhow::Result;
use clap::{Parser, Subcommand};

use run_bob::commands;

#[derive(Parser, Debug)]
#[command(name = "run-bob")]
#[command(about = "Bootstrap Bob's 4-ring Clean Architecture + Superpowers harness for Claude Code and Codex projects", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize the run-bob harness in the current directory
    Init {
        /// Overwrite existing files
        #[arg(short, long)]
        force: bool,

        /// Only install skills, skip CLAUDE.md / ARCHITECTURE.md / README / shared / ArchUnit
        #[arg(short, long)]
        minimal: bool,

        /// Target directory (default: current directory)
        #[arg(short, long, default_value = ".")]
        dir: String,

        /// Skip writing the run-bob block into the target directory's .gitignore
        #[arg(long)]
        no_gitignore: bool,

        /// Also install the Java/Maven skeleton (ArchUnit test + shared UseCase /
        /// TransactionalUseCaseDecorator under src/). Off by default — only useful
        /// when the target project is a Java/Spring project ready to enforce the
        /// 4-ring architecture at test time.
        #[arg(long)]
        with_java: bool,
    },

    /// Check if the current project has the run-bob harness properly installed
    Status {
        /// Target directory (default: current directory)
        #[arg(short, long, default_value = ".")]
        dir: String,
    },

    /// Re-sync upgrade-safe harness assets in a target project with the current run-bob binary
    Upgrade {
        /// Target directory (default: current directory)
        #[arg(short, long, default_value = ".")]
        dir: String,

        /// Only report what would change; do not write any files
        #[arg(short = 'n', long)]
        dry_run: bool,

        /// Skip the safety backup before overwriting (dangerous)
        #[arg(long)]
        no_backup: bool,

        /// Skip writing the run-bob block into the target directory's .gitignore
        #[arg(long)]
        no_gitignore: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init {
            force,
            minimal,
            dir,
            no_gitignore,
            with_java,
        } => {
            commands::init::run(&dir, force, minimal, no_gitignore, with_java)?;
        }
        Commands::Status { dir } => {
            commands::status::run(&dir)?;
        }
        Commands::Upgrade {
            dir,
            dry_run,
            no_backup,
            no_gitignore,
        } => {
            commands::upgrade::run(&dir, dry_run, no_backup, no_gitignore)?;
        }
    }

    Ok(())
}
