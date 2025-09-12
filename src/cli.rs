use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum OutputFormat {
    Plain,
    Json,
    Ndjson,
    Ai,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum GraphFormat {
    Mermaid,
    Dot,
    Json,
}

#[derive(Parser, Debug)]
#[command(
    name = "cli-rag",
    version,
    about = "Per-repo ADR navigator with TOML config"
)]
pub struct Cli {
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,

    #[arg(long, value_delimiter = ',', global = true, alias = "filepaths")]
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
        /// Optional path to write config (defaults to ./.cli-rag.toml)
        #[arg(long)]
        path: Option<PathBuf>,
        #[arg(long)]
        force: bool,
        #[arg(long)]
        print_template: bool,
        /// Do not open the config in an editor after creating or detecting it
        #[arg(long, default_value_t = false)]
        silent: bool,
        /// Optional schema name to scaffold into the config
        #[arg(long)]
        schema: Option<String>,
        /// Write schema to a separate file under .cli-rag/templates/ and add to import
        #[arg(long, default_value_t = false)]
        separate: bool,
    },
    Info {},
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
    /// Create a new note from a schema template
    New {
        /// Schema name to use (e.g., ADR, IMP)
        #[arg(long)]
        schema: String,
        /// Optional title for the note (used in template)
        #[arg(long)]
        title: Option<String>,
        /// Optional filename template, e.g. "{{id}}-{{title}}.md"
        #[arg(long, value_name = "TEMPLATE")]
        filename_template: Option<String>,
        /// Destination base directory to write the note (must match a configured base)
        #[arg(long, value_name = "PATH")]
        dest_base: Option<std::path::PathBuf>,
        /// Normalize title to Title Case before rendering
        #[arg(long, default_value_t = false)]
        normalize_title: bool,
        /// Print rendered body only; do not write a file
        #[arg(long, default_value_t = false)]
        print_body: bool,
        /// Do not write; print what would be done
        #[arg(long, default_value_t = false)]
        dry_run: bool,
        /// Open the created file in EDITOR/visual
        #[arg(long, default_value_t = false)]
        edit: bool,
    },
    /// Export a dependency graph (mermaid|dot|json)
    Graph {
        #[arg(long)]
        id: String,
        #[arg(long)]
        depth: Option<usize>,
        #[arg(long)]
        include_bidirectional: Option<bool>,
        /// Output format (json is the machine/AI surface)
        #[arg(long = "graph-format", value_enum, default_value_t = GraphFormat::Mermaid, help = "Output format (json is the machine/AI surface)")]
        graph_format: GraphFormat,
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
        /// Emit ACP-like NDJSON events to stdout
        #[arg(long, default_value_t = false)]
        json: bool,
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
