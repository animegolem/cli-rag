use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum OutputFormat {
    Plain,
    Json,
    Ndjson,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum GraphFormat {
    Mermaid,
    Dot,
    Json,
}

#[derive(Parser, Debug)]
#[command(
    name = "adr-rag",
    version,
    about = "Per-repo ADR navigator with TOML config"
)]
pub struct Cli {
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,

    #[arg(long, value_delimiter = ',', global = true)]
    pub base: Option<Vec<PathBuf>>,

    /// Global output format
    #[arg(long, value_enum, global = true, default_value_t = OutputFormat::Plain)]
    pub format: OutputFormat,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Init {
        /// Optional path to write config (defaults to ./.adr-rag.toml)
        #[arg(long)]
        path: Option<PathBuf>,
        #[arg(long)]
        force: bool,
        #[arg(long)]
        print_template: bool,
        /// Do not open the config in an editor after creating or detecting it
        #[arg(long, default_value_t = false)]
        silent: bool,
    },
    Doctor {},
    Search {
        #[arg(long, short = 'q')]
        query: String,
    },
    Topics {},
    Group {
        #[arg(long)]
        topic: String,
        #[arg(long)]
        include_content: Option<bool>,
    },
    Get {
        #[arg(long)]
        id: String,
        #[arg(long, default_value_t = false)]
        include_dependents: bool,
    },
    Cluster {
        #[arg(long)]
        id: String,
        #[arg(long)]
        depth: Option<usize>,
        #[arg(long)]
        include_bidirectional: Option<bool>,
    },
    Path {
        #[arg(long, value_name = "FROM")]
        from: String,
        #[arg(long, value_name = "TO")]
        to: String,
        #[arg(long, default_value_t = 5)]
        max_depth: usize,
    },
    /// Export a dependency graph (mermaid|dot|json)
    Graph {
        #[arg(long)]
        id: String,
        #[arg(long)]
        depth: Option<usize>,
        #[arg(long)]
        include_bidirectional: Option<bool>,
        /// Output format
        #[arg(long, value_enum, default_value_t = GraphFormat::Mermaid)]
        format: GraphFormat,
    },
    Validate(ValidateArgs),

    /// Watch bases and incrementally validate + update indexes on changes
    Watch {
        /// Force full rescan on first run
        #[arg(long, default_value_t = false)]
        full_rescan: bool,
        /// Debounce milliseconds for coalescing FS events
        #[arg(long, default_value_t = 400)]
        debounce_ms: u64,
        /// Print only; do not write indexes or groups
        #[arg(long, default_value_t = false)]
        dry_run: bool,
    },

    /// Generate shell completions (bash|zsh|fish)
    Completions {
        #[arg(value_name = "SHELL")]
        shell: String,
    },
}

#[derive(Args, Debug)]
pub struct ValidateArgs {
    #[arg(long, value_enum, default_value_t = OutputFormat::Plain)]
    pub format: OutputFormat,
    #[arg(long, default_value_t = false)]
    pub write_groups: bool,
    /// Do not write index/groups; print results only
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,
    /// Force full rescan instead of incremental
    #[arg(long, default_value_t = false)]
    pub full_rescan: bool,
}
