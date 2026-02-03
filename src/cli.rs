use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "dwf", version, about = "Developer Workflow Fastlane (Rust)")]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Create a default dwf.toml in the current directory
    Init {
        /// Overwrite existing dwf.toml
        #[arg(long)]
        force: bool,
    },

    /// Run the workflow pipeline (fail-fast)
    Run {
        /// Pipeline mode: fast or full
        #[arg(value_enum, default_value = "fast")]
        mode: Mode,

        /// Do not write to .dwf/history.jsonl
        #[arg(long)]
        no_save: bool,
    },

    /// Print a summary report from recent runs
    Report {
        /// Number of recent runs to include
        #[arg(long, default_value_t = 10)]
        last: usize,
    },

    /// Suggest improvements based on environment and recent timings
    Doctor,
}

#[derive(Copy, Clone, Debug, ValueEnum, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum Mode {
    Fast,
    Full,
}
