mod cli;
mod config;
mod doctor;
mod metrics;
mod report;
mod runner;
mod templates;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let args = cli::Args::parse();

    match args.command {
        cli::Command::Run { mode, no_save } => {
            let cfg = config::load_config()?;
            let run = runner::run_pipeline(&cfg, mode)?;
            runner::print_run_summary(&run);

            if !no_save {
                metrics::append_run(&run)?;
            }

            // Exit non-zero if pipeline failed (useful for CI)
            if !run.ok {
                std::process::exit(1);
            }
        }
        cli::Command::Report { last } => {
            let runs = metrics::load_last_runs(last)?;
            report::print_report(&runs, last);
        }
        cli::Command::Doctor => {
            let cfg = config::load_config()?;
            let runs = metrics::load_last_runs(20).unwrap_or_default();
            doctor::print_doctor(&cfg, &runs);
        }
        cli::Command::Examples => {
            for t in templates::list_templates() {
                println!("{}", t);
            }
        }
        cli::Command::Init { force, template } => {
            let cfg = templates::get_template(&template)
                .ok_or_else(|| anyhow::anyhow!("Unknown template: {}", template))?;
            config::init_config_with(cfg, force)?;
            println!("Initialized dwf.toml (template: {})", template);
        }
    }

    Ok(())
}
