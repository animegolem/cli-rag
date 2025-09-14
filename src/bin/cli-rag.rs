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
        Commands::Info {} => {
            let (cfg, cfg_path) = load_config(&cli.config, &cli.base, cli.no_lua)?;
            cli_rag::commands::info::run(&cfg, &cfg_path, &cli.format)?;
        }
        Commands::Completions { shell } => {
            let cmd = Cli::command();
            cli_rag::commands::completions::run_completions(cmd, shell);
        }
        Commands::Search {
            query,
            kind,
            schema,
            status,
            tag,
        } => {
            let (cfg, cfg_path) = load_config(&cli.config, &cli.base, cli.no_lua)?;
            cli_rag::commands::search::run(
                &cfg,
                &cfg_path,
                &cli.format,
                query,
                kind,
                schema,
                status,
                tag,
            )?;
        }

        Commands::Get {
            id,
            include_dependents,
            neighbor_style,
            depth,
            max_fanout,
        } => {
            let (cfg, cfg_path) = load_config(&cli.config, &cli.base, cli.no_lua)?;
            cli_rag::commands::get::run(
                &cfg,
                &cfg_path,
                &cli.format,
                id,
                include_dependents,
                neighbor_style,
                depth,
                max_fanout,
            )?;
        }
        Commands::Cluster {
            id,
            depth,
            include_bidirectional,
        } => {
            let (cfg, cfg_path) = load_config(&cli.config, &cli.base, cli.no_lua)?;
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
            let (cfg, cfg_path) = load_config(&cli.config, &cli.base, cli.no_lua)?;
            cli_rag::commands::path::run(&cfg, &cfg_path, &cli.format, from, to, max_depth)?;
        }
        Commands::Graph {
            id,
            depth,
            include_bidirectional,
            graph_format,
        } => {
            let (cfg, cfg_path) = load_config(&cli.config, &cli.base, cli.no_lua)?;
            cli_rag::commands::graph::run(
                &cfg,
                &cfg_path,
                &graph_format,
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
            normalize_title,
            print_body,
            dry_run,
            edit,
        } => {
            let (cfg, cfg_path) = load_config(&cli.config, &cli.base, cli.no_lua)?;
            cli_rag::commands::new::run(
                &cfg,
                &cfg_path,
                schema,
                title,
                filename_template,
                dest_base,
                normalize_title,
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
            let (cfg, cfg_path) = load_config(&cli.config, &cli.base, cli.no_lua)?;
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
            let (cfg, cfg_path) = load_config(&cli.config, &cli.base, cli.no_lua)?;
            cli_rag::commands::validate_cmd::run(
                &cfg,
                &cfg_path,
                &args.format,
                args.dry_run,
                args.full_rescan,
            )?;
        }
        Commands::AiIndexPlan {
            edges,
            min_cluster_size,
            schema,
            output,
        } => {
            let (cfg, cfg_path) = load_config(&cli.config, &cli.base, cli.no_lua)?;
            cli_rag::commands::ai_index_plan::run(
                &cfg,
                &cfg_path,
                edges,
                min_cluster_size,
                schema,
                output,
            )?;
        }
        Commands::AiIndexApply {
            from,
            write_cache,
            write_frontmatter,
            dry_run,
        } => {
            let (cfg, cfg_path) = load_config(&cli.config, &cli.base, cli.no_lua)?;
            cli_rag::commands::ai_index_apply::run(
                &cfg,
                &cfg_path,
                from,
                write_cache,
                write_frontmatter,
                dry_run,
            )?;
        }
    }
    Ok(())
}
