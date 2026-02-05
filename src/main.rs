mod checker;
mod cli;
mod error;
mod highlight;
mod hints;
mod i18n;
mod linter;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command};

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize i18n based on locale or CLI flag
    let lang = cli.lang.as_deref().unwrap_or_else(|| {
        if i18n::is_japanese_locale() {
            "ja"
        } else {
            "en"
        }
    });
    let messages = i18n::Messages::new(lang);

    match cli.command {
        Command::Check { paths, dialect } => {
            checker::check(&paths, &dialect, &messages)?;
        }
        Command::Fix {
            paths,
            dialect,
            dry_run,
            format,
        } => {
            checker::fix(&paths, &dialect, dry_run, format, &messages)?;
        }
        Command::Lint {
            paths,
            dialect,
            keyword_case,
            no_select_star,
            require_alias,
        } => {
            checker::lint(
                &paths,
                &dialect,
                &keyword_case,
                no_select_star,
                require_alias,
                &messages,
            )?;
        }
    }

    Ok(())
}
