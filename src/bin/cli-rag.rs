use anyhow::Result;
use clap::{CommandFactory, Parser};

use cli_rag::cli::{
    AiCommands, AiIndexApplyArgs, AiIndexCommands, AiIndexPlanArgs, AiNewCommands, AiNewSubmitArgs,
    Cli, Commands,
};
use cli_rag::commands::ai_new::{SubmitInput, SubmitRequest};
use cli_rag::config::load_config;

fn warn_deprecated_alias(subcommand: &str) {
    eprintln!(
        "Deprecated: use `cli-rag ai index {}` (alias will be removed in a future release)",
        subcommand
    );
}

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
        Commands::AiIndexPlan(args) => {
            warn_deprecated_alias("plan");
            let AiIndexPlanArgs {
                edges,
                min_cluster_size,
                schema,
                output,
            } = args;
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
        Commands::AiIndexApply(args) => {
            warn_deprecated_alias("apply");
            let AiIndexApplyArgs {
                from,
                write_cache,
                write_frontmatter,
                dry_run,
            } = args;
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
        Commands::Ai { command } => match command {
            AiCommands::New { command } => match command {
                AiNewCommands::Start(args) => {
                    let (cfg, cfg_path) = load_config(&cli.config, &cli.base, cli.no_lua)?;
                    cli_rag::commands::ai_new::start(
                        &cfg,
                        &cfg_path,
                        args.schema,
                        args.title,
                        args.id,
                        &cli.format,
                    )?;
                }
                AiNewCommands::Submit(args) => {
                    let (cfg, cfg_path) = load_config(&cli.config, &cli.base, cli.no_lua)?;
                    let AiNewSubmitArgs {
                        draft,
                        stdin,
                        sections,
                        from_file,
                        allow_oversize,
                    } = args;
                    let input = match (stdin, sections, from_file) {
                        (true, _, _) => SubmitInput::Stdin,
                        (false, Some(path), _) => SubmitInput::Sections(path),
                        (false, None, Some(path)) => SubmitInput::Markdown(path),
                        _ => unreachable!("clap guarantees one submit input"),
                    };
                    let request = SubmitRequest {
                        draft_id: draft,
                        input,
                        allow_oversize,
                    };
                    cli_rag::commands::ai_new::submit(&cfg, &cfg_path, request, &cli.format)?;
                }
                AiNewCommands::Cancel(args) => {
                    let (cfg, cfg_path) = load_config(&cli.config, &cli.base, cli.no_lua)?;
                    cli_rag::commands::ai_new::cancel(&cfg, &cfg_path, args.draft, &cli.format)?;
                }
                AiNewCommands::List(args) => {
                    let (cfg, cfg_path) = load_config(&cli.config, &cli.base, cli.no_lua)?;
                    cli_rag::commands::ai_new::list(&cfg, &cfg_path, args.stale_days, &cli.format)?;
                }
            },
            AiCommands::Index { command } => match command {
                AiIndexCommands::Plan(args) => {
                    let (cfg, cfg_path) = load_config(&cli.config, &cli.base, cli.no_lua)?;
                    let AiIndexPlanArgs {
                        edges,
                        min_cluster_size,
                        schema,
                        output,
                    } = args;
                    cli_rag::commands::ai_index_plan::run(
                        &cfg,
                        &cfg_path,
                        edges,
                        min_cluster_size,
                        schema,
                        output,
                    )?;
                }
                AiIndexCommands::Apply(args) => {
                    let (cfg, cfg_path) = load_config(&cli.config, &cli.base, cli.no_lua)?;
                    let AiIndexApplyArgs {
                        from,
                        write_cache,
                        write_frontmatter,
                        dry_run,
                    } = args;
                    cli_rag::commands::ai_index_apply::run(
                        &cfg,
                        &cfg_path,
                        from,
                        write_cache,
                        write_frontmatter,
                        dry_run,
                    )?;
                }
            },
        },
    }
    Ok(())
}
