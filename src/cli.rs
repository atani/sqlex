use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum FixFormat {
    /// Summary of changes (default)
    #[default]
    Summary,
    /// Unified diff format
    Diff,
}

#[derive(Parser)]
#[command(name = "sqlex")]
#[command(about = "SQL syntax checker and linter", long_about = None)]
#[command(version)]
pub struct Cli {
    /// Language for messages (en, ja)
    #[arg(long, global = true)]
    pub lang: Option<String>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Check SQL files for syntax errors
    Check {
        /// Files or directories to check
        #[arg(required = true)]
        paths: Vec<String>,

        /// SQL dialect (generic, mysql, postgres, sqlite, bigquery)
        #[arg(short, long, default_value = "generic")]
        dialect: String,
    },

    /// Fix SQL files automatically
    Fix {
        /// Files or directories to fix
        #[arg(required = true)]
        paths: Vec<String>,

        /// SQL dialect (generic, mysql, postgres, sqlite, bigquery)
        #[arg(short, long, default_value = "generic")]
        dialect: String,

        /// Show what would be changed without modifying files
        #[arg(long)]
        dry_run: bool,

        /// Output format for dry-run (summary, diff)
        #[arg(short, long, default_value = "summary")]
        format: FixFormat,
    },

    /// Lint SQL files for style issues
    Lint {
        /// Files or directories to lint
        #[arg(required = true)]
        paths: Vec<String>,

        /// SQL dialect (generic, mysql, postgres, sqlite, bigquery)
        #[arg(short, long, default_value = "generic")]
        dialect: String,

        /// Keyword case style (upper, lower, ignore)
        #[arg(long, default_value = "upper")]
        keyword_case: String,

        /// Disallow SELECT *
        #[arg(long, default_value = "true")]
        no_select_star: bool,

        /// Require table aliases
        #[arg(long)]
        require_alias: bool,
    },
}
