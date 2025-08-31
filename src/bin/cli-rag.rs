use anyhow::Result;
use clap::{CommandFactory, Parser};

use cli_rag::cli::{Cli, Commands};
use cli_rag::config::load_config;

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init {
            path,
            force,
            print_template,
            silent,
            schema,
            separate,
        } => {
            cli_rag::commands::init::run(path, force, print_template, silent, schema, separate)?;
        }
        Commands::Doctor {} => {
            let (cfg, cfg_path) = load_config(&cli.config, &cli.base)?;
            cli_rag::commands::doctor::run(&cfg, &cfg_path, &cli.format)?;
        }
        Commands::Completions { shell } => {
            let cmd = Cli::command();
            cli_rag::commands::completions::run_completions(cmd, shell);
        }
        Commands::Search { query } => {
            let (cfg, cfg_path) = load_config(&cli.config, &cli.base)?;
            cli_rag::commands::search::run(&cfg, &cfg_path, &cli.format, query)?;
        }
        Commands::Topics {} => {
            let (cfg, cfg_path) = load_config(&cli.config, &cli.base)?;
            cli_rag::commands::topics::run(&cfg, &cfg_path, &cli.format)?;
        }
        Commands::Group {
            topic,
            include_content,
        } => {
            let (cfg, cfg_path) = load_config(&cli.config, &cli.base)?;
            cli_rag::commands::group::run(&cfg, &cfg_path, &cli.format, topic, include_content)?;
        }
        Commands::Get {
            id,
            include_dependents,
        } => {
            let (cfg, cfg_path) = load_config(&cli.config, &cli.base)?;
            cli_rag::commands::get::run(&cfg, &cfg_path, &cli.format, id, include_dependents)?;
        }
        Commands::Cluster {
            id,
            depth,
            include_bidirectional,
        } => {
            let (cfg, cfg_path) = load_config(&cli.config, &cli.base)?;
            cli_rag::commands::cluster::run(
                &cfg,
                &cfg_path,
                &cli.format,
                id,
                depth,
                include_bidirectional,
            )?;
        }
        Commands::Path {
            from,
            to,
            max_depth,
        } => {
            let (cfg, cfg_path) = load_config(&cli.config, &cli.base)?;
            cli_rag::commands::path::run(&cfg, &cfg_path, &cli.format, from, to, max_depth)?;
        }
        Commands::Graph {
            id,
            depth,
            include_bidirectional,
            format,
        } => {
            let (cfg, cfg_path) = load_config(&cli.config, &cli.base)?;
            cli_rag::commands::graph::run(
                &cfg,
                &cfg_path,
                &format,
                id,
                depth,
                include_bidirectional,
            )?;
        }
        Commands::New {
            schema,
            title,
            filename_template,
            dest_base,
            print_body,
            dry_run,
            edit,
        } => {
            let (cfg, cfg_path) = load_config(&cli.config, &cli.base)?;
            cli_rag::commands::new::run(
                &cfg,
                &cfg_path,
                schema,
                title,
                filename_template,
                dest_base,
                print_body,
                dry_run,
                edit,
            )?;
        }
        Commands::Watch {
            full_rescan,
            debounce_ms,
            dry_run,
            json,
        } => {
            let (cfg, cfg_path) = load_config(&cli.config, &cli.base)?;
            cli_rag::commands::watch_cmd::run(
                &cfg,
                &cfg_path,
                full_rescan,
                debounce_ms,
                dry_run,
                json,
            )?;
        }
        Commands::Validate(args) => {
            let (cfg, cfg_path) = load_config(&cli.config, &cli.base)?;
            cli_rag::commands::validate_cmd::run(
                &cfg,
                &cfg_path,
                &args.format,
                args.write_groups,
                args.dry_run,
                args.full_rescan,
            )?;
        }
    }
    Ok(())
}
