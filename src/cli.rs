use clap::{ArgGroup, Args, Parser, Subcommand, ValueEnum};
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

    /// Disable Lua overlays entirely (also honored via CLI_RAG_NO_LUA=1)
    #[arg(long, global = true, default_value_t = false)]
    pub no_lua: bool,

    /// Global output format
    #[arg(long, value_enum, global = true, default_value_t = OutputFormat::Plain)]
    pub format: OutputFormat,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Scaffold a project config and optional schema templates
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
        /// Choose init preset non-interactively (e.g., project, generic, example)
        #[arg(long)]
        preset: Option<String>,
        /// Preview generated files without writing them
        #[arg(long, default_value_t = false)]
        dry_run: bool,
        /// Emit JSON summary of created/updated files
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// Inspect resolved config, cache entries, and capabilities
    Info {},
    /// Search notes with fuzzy text matching and filters
    Search {
        #[arg(long, short = 'q')]
        query: String,
        /// Filter by item kind (note,todo,kanban)
        #[arg(long, value_delimiter = ',')]
        kind: Option<Vec<String>>,
        /// Filter by schema name(s)
        #[arg(long, value_delimiter = ',')]
        schema: Option<Vec<String>>,
        /// Filter by status value(s)
        #[arg(long, value_delimiter = ',')]
        status: Option<Vec<String>>,
        /// Filter by tag(s)
        #[arg(long, value_delimiter = ',')]
        tag: Option<Vec<String>>,
    },
    /// Retrieve a note with its neighborhood for AI workflows
    Get {
        #[arg(long)]
        id: String,
        #[arg(long, default_value_t = false)]
        include_dependents: bool,
        /// Neighbor style for JSON output (metadata|outline|full)
        #[arg(long, value_name = "STYLE")]
        neighbor_style: Option<String>,
        /// Neighbor search depth (JSON)
        #[arg(long)]
        depth: Option<usize>,
        /// Max neighbors to include (JSON)
        #[arg(long, value_name = "N")]
        max_fanout: Option<usize>,
    },
    /// Explore dependency clusters around a given note
    Cluster {
        #[arg(long)]
        id: String,
        #[arg(long)]
        depth: Option<usize>,
        #[arg(long)]
        include_bidirectional: Option<bool>,
    },
    /// Compute the shortest dependency path between notes
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
        /// Output format (json is the machine/AI surface)
        #[arg(long = "graph-format", value_enum, default_value_t = GraphFormat::Mermaid, help = "Output format (json is the machine/AI surface)")]
        graph_format: GraphFormat,
    },
    /// Build or print the unified index and run validation checks
    Validate(ValidateArgs),

    /// Watch bases and incrementally validate + update indexes on changes
    Watch {
        /// Force full rescan on first run
        #[arg(long, default_value_t = false)]
        full_rescan: bool,
        /// Debounce milliseconds for coalescing FS events
        #[arg(long, default_value_t = 400)]
        debounce_ms: u64,
        /// Print only; do not write index
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
    /// AI-oriented workflows (authoring, retrieval, indexing)
    Ai {
        #[command(subcommand)]
        command: AiCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum AiCommands {
    /// AI authoring helpers for managed drafts (start/submit/cancel/list)
    New {
        #[command(subcommand)]
        command: AiNewCommands,
    },
    /// AI index workflows (plan/apply)
    Index {
        #[command(subcommand)]
        command: AiIndexCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum AiNewCommands {
    /// Start an AI draft, reserving ID/filename and returning constraints
    Start(AiNewStartArgs),
    /// Submit an AI draft with rendered sections/frontmatter to finalize the note
    Submit(AiNewSubmitArgs),
    /// Cancel an AI draft and release its reservation
    Cancel(AiNewCancelArgs),
    /// List active drafts (optionally filtering by staleness)
    List(AiNewListArgs),
}

#[derive(Subcommand, Debug)]
pub enum AiIndexCommands {
    /// Compute AI index plan over the unified graph and write JSON
    Plan(AiIndexPlanArgs),
    /// Apply an AI index plan: write cache and optionally add tags
    Apply(AiIndexApplyArgs),
}

#[derive(Args, Debug)]
pub struct AiNewStartArgs {
    /// Schema name to use (e.g., ADR, IMP)
    #[arg(long)]
    pub schema: String,
    /// Optional title to seed template and filename rendering
    #[arg(long)]
    pub title: Option<String>,
    /// Optional explicit ID override; otherwise engine assigns via schema rules
    #[arg(long)]
    pub id: Option<String>,
}

#[derive(Args, Debug)]
#[command(group = ArgGroup::new("submit_input").args(["stdin", "sections", "from_file"]).required(true))]
pub struct AiNewSubmitArgs {
    /// Draft identifier returned by `ai new start`
    #[arg(long)]
    pub draft: String,
    /// Read structured JSON payload from stdin
    #[arg(long, default_value_t = false)]
    pub stdin: bool,
    /// Path to structured JSON payload file ({frontmatter, sections})
    #[arg(long, value_name = "PATH")]
    pub sections: Option<std::path::PathBuf>,
    /// Path to Markdown note to parse into sections
    #[arg(long = "from-file", value_name = "PATH")]
    pub from_file: Option<std::path::PathBuf>,
    /// Allow writing even if line-count constraints fail (marks needs_attention)
    #[arg(long, default_value_t = false)]
    pub allow_oversize: bool,
}

#[derive(Args, Debug)]
pub struct AiNewCancelArgs {
    /// Optional draft identifier to cancel (omit to auto-select when only one draft exists)
    #[arg(long)]
    pub draft: Option<String>,
}

#[derive(Args, Debug)]
pub struct AiNewListArgs {
    /// Optional staleness filter in days (show only drafts older than N days)
    #[arg(long, value_name = "DAYS")]
    pub stale_days: Option<u64>,
}

#[derive(Args, Debug)]
pub struct AiIndexPlanArgs {
    /// Comma-separated list of edge kinds to include (default: depends_on,mentions)
    #[arg(long, value_delimiter = ',')]
    pub edges: Option<Vec<String>>,
    /// Minimum cluster size to include (default: 3)
    #[arg(long, default_value_t = 3)]
    pub min_cluster_size: usize,
    /// Optional schema filter (only include nodes of this schema)
    #[arg(long)]
    pub schema: Option<String>,
    /// Output path for the plan JSON
    #[arg(long, value_name = "PATH")]
    pub output: std::path::PathBuf,
}

#[derive(Args, Debug)]
pub struct AiIndexApplyArgs {
    /// Path to the plan JSON generated by ai index plan
    #[arg(long, value_name = "PATH")]
    pub from: std::path::PathBuf,
    /// Write authoritative cache (.cli-rag/cache/ai-index.json)
    #[arg(long, default_value_t = true)]
    pub write_cache: bool,
    /// Write tags to frontmatter (additive)
    #[arg(long, default_value_t = false)]
    pub write_frontmatter: bool,
    /// Dry run; do not write files
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,
}

#[derive(Args, Debug)]
pub struct ValidateArgs {
    #[arg(long, value_enum, default_value_t = OutputFormat::Plain)]
    pub format: OutputFormat,
    /// Do not write index; print results only
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,
    /// Force full rescan instead of incremental
    #[arg(long, default_value_t = false)]
    pub full_rescan: bool,
}
