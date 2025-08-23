use anyhow::Result;
use clap::{Parser, CommandFactory};

use adr_rag::cli::{Cli, Commands};
use adr_rag::config::load_config;

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init { path, force, print_template, silent } => {
            adr_rag::commands::init::run(path, force, print_template, silent)?;
        }
        Commands::Doctor {} => {
            let (cfg, cfg_path) = load_config(&cli.config, &cli.base)?;
            adr_rag::commands::doctor::run(&cfg, &cfg_path, &cli.format)?;
        }
        Commands::Completions { shell } => {
            let cmd = Cli::command();
            adr_rag::commands::completions::run_completions(cmd, shell);
        }
        Commands::Search { query } => {
            let (cfg, _) = load_config(&cli.config, &cli.base)?;
            adr_rag::commands::search::run(&cfg, &cli.format, query)?;
        }
        Commands::Topics {} => {
            let (cfg, _) = load_config(&cli.config, &cli.base)?;
            adr_rag::commands::topics::run(&cfg, &cli.format)?;
        }
        Commands::Group { topic, include_content } => {
            let (cfg, _) = load_config(&cli.config, &cli.base)?;
            adr_rag::commands::group::run(&cfg, &cli.format, topic, include_content)?;
        }
        Commands::Get { id, include_dependents } => {
            let (cfg, _) = load_config(&cli.config, &cli.base)?;
            adr_rag::commands::get::run(&cfg, &cli.format, id, include_dependents)?;
        }
        Commands::Cluster { id, depth, include_bidirectional } => {
            let (cfg, _) = load_config(&cli.config, &cli.base)?;
            adr_rag::commands::cluster::run(&cfg, &cli.format, id, depth, include_bidirectional)?;
        }
        Commands::Path { from, to, max_depth } => {
            let (cfg, _) = load_config(&cli.config, &cli.base)?;
            adr_rag::commands::path::run(&cfg, &cli.format, from, to, max_depth)?;
        }
        Commands::Graph { id, depth, include_bidirectional, format } => {
            let (cfg, _) = load_config(&cli.config, &cli.base)?;
            adr_rag::commands::graph::run(&cfg, &format, id, depth, include_bidirectional)?;
        }
        Commands::Watch { full_rescan, debounce_ms, dry_run } => {
            let (cfg, _) = load_config(&cli.config, &cli.base)?;
            adr_rag::commands::watch_cmd::run(&cfg, full_rescan, debounce_ms, dry_run)?;
        }
        Commands::Validate(args) => {
            let (cfg, _) = load_config(&cli.config, &cli.base)?;
            adr_rag::commands::validate_cmd::run(&cfg, &args.format, args.write_groups, args.dry_run, args.full_rescan)?;
        }
    }
    Ok(())
}
