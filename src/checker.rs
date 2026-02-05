use crate::cli::FixFormat;
use crate::error::SqlexError;
use crate::highlight::SourceHighlighter;
use crate::hints;
use crate::i18n::Messages;
use crate::linter::{KeywordCase, LintConfig, Linter};
use anyhow::{Context, Result};
use colored::Colorize;
use similar::{ChangeTag, TextDiff};
use sqlparser::dialect::{
    BigQueryDialect, Dialect, GenericDialect, MySqlDialect, PostgreSqlDialect, SQLiteDialect,
};
use sqlparser::parser::Parser;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

fn get_dialect(name: &str) -> Result<Box<dyn Dialect>> {
    match name.to_lowercase().as_str() {
        "generic" => Ok(Box::new(GenericDialect {})),
        "mysql" => Ok(Box::new(MySqlDialect {})),
        "postgres" | "postgresql" => Ok(Box::new(PostgreSqlDialect {})),
        "sqlite" => Ok(Box::new(SQLiteDialect {})),
        "bigquery" => Ok(Box::new(BigQueryDialect {})),
        _ => Err(SqlexError::UnsupportedDialect(name.to_string()).into()),
    }
}

fn collect_sql_files(paths: &[String]) -> Vec<String> {
    let mut files = Vec::new();

    for path in paths {
        let p = Path::new(path);
        if p.is_file() && path.ends_with(".sql") {
            files.push(path.clone());
        } else if p.is_dir() {
            for entry in WalkDir::new(p).into_iter().filter_map(|e| e.ok()) {
                let entry_path = entry.path();
                if entry_path.is_file() && entry_path.extension().is_some_and(|ext| ext == "sql") {
                    files.push(entry_path.to_string_lossy().to_string());
                }
            }
        }
    }

    files
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct CheckResult {
    pub path: String,
    pub errors: Vec<SyntaxError>,
}

#[derive(Debug)]
pub struct SyntaxError {
    pub line: usize,
    pub column: usize,
    pub message: String,
}

fn parse_error_location(error_msg: &str) -> (usize, usize) {
    // sqlparser error format: "... at Line: X, Column: Y" or "... at Line: X, Column Y"
    let line = error_msg
        .find("Line: ")
        .and_then(|i| {
            let start = i + 6;
            let rest = &error_msg[start..];
            let end = rest
                .find(|c: char| !c.is_ascii_digit())
                .unwrap_or(rest.len());
            rest[..end].parse().ok()
        })
        .unwrap_or(1);

    // Handle both "Column: X" and "Column X" formats
    let column = error_msg
        .find("Column")
        .and_then(|i| {
            let rest = &error_msg[i + 6..]; // Skip "Column"
                                            // Skip any non-digit characters (colon, space)
            let start = rest.find(|c: char| c.is_ascii_digit())?;
            let num_rest = &rest[start..];
            let end = num_rest
                .find(|c: char| !c.is_ascii_digit())
                .unwrap_or(num_rest.len());
            num_rest[..end].parse().ok()
        })
        .unwrap_or(1);

    (line, column)
}

fn check_sql(content: &str, dialect: &dyn Dialect) -> Vec<SyntaxError> {
    match Parser::parse_sql(dialect, content) {
        Ok(_) => vec![],
        Err(e) => {
            let msg = e.to_string();
            let (line, column) = parse_error_location(&msg);
            vec![SyntaxError {
                line,
                column,
                message: msg,
            }]
        }
    }
}

pub fn check(paths: &[String], dialect_name: &str, messages: &Messages) -> Result<()> {
    let dialect = get_dialect(dialect_name)?;
    let files = collect_sql_files(paths);

    if files.is_empty() {
        eprintln!("{}", "No SQL files found".yellow());
        return Ok(());
    }

    let mut total_errors = 0;
    let mut results = Vec::new();

    for file in &files {
        let content =
            fs::read_to_string(file).with_context(|| format!("Failed to read: {}", file))?;

        let errors = check_sql(&content, dialect.as_ref());

        if errors.is_empty() {
            println!("{}", messages.file_ok(file).green());
        } else {
            println!("{}", messages.file_error(file, errors.len()).red());
            for error in &errors {
                println!(
                    "  {}",
                    messages.syntax_error(error.line, error.column, &error.message)
                );

                // Analyze error and provide hints
                let hint = hints::analyze_error(&error.message, &content, error.line, messages);

                if let Some(ref h) = hint {
                    println!("  {} {}", "💡".yellow(), h.hint.yellow());
                }

                // Display highlighted source code with suspect line
                let suspect_line = hint.and_then(|h| h.suspect_line);
                let highlight = SourceHighlighter::display_error_with_hint(
                    &content,
                    error.line,
                    error.column,
                    suspect_line,
                    2,
                );
                println!("{}", highlight);
                println!();
            }
            total_errors += errors.len();
        }

        results.push(CheckResult {
            path: file.clone(),
            errors,
        });
    }

    println!("{}", messages.summary(files.len(), total_errors));

    if total_errors > 0 {
        std::process::exit(1);
    }

    Ok(())
}

pub fn fix(
    paths: &[String],
    dialect_name: &str,
    dry_run: bool,
    format: FixFormat,
    messages: &Messages,
) -> Result<()> {
    let dialect = get_dialect(dialect_name)?;
    let files = collect_sql_files(paths);

    if files.is_empty() {
        eprintln!("{}", "No SQL files found".yellow());
        return Ok(());
    }

    for file in &files {
        let content =
            fs::read_to_string(file).with_context(|| format!("Failed to read: {}", file))?;

        // Parse and reformat
        match Parser::parse_sql(dialect.as_ref(), &content) {
            Ok(ast) => {
                let formatted: Vec<String> = ast.iter().map(|stmt| stmt.to_string()).collect();
                let new_content = formatted.join(";\n") + ";\n";

                if new_content != content {
                    if dry_run {
                        match format {
                            FixFormat::Summary => {
                                println!("{}", messages.would_fix(file).yellow());
                                print_summary_diff(&content, &new_content);
                            }
                            FixFormat::Diff => {
                                print_unified_diff(file, &content, &new_content);
                            }
                        }
                    } else {
                        fs::write(file, &new_content)
                            .with_context(|| format!("Failed to write: {}", file))?;
                        println!("{}", messages.fixed(file).green());
                    }
                }
            }
            Err(e) => {
                println!("{}", messages.file_error(file, 1).red());
                println!("  Cannot fix file with syntax errors: {}", e);
            }
        }
    }

    Ok(())
}

fn print_summary_diff(old: &str, new: &str) {
    let diff = TextDiff::from_lines(old, new);
    let mut line_num = 0;

    for change in diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Equal => {
                line_num += 1;
            }
            ChangeTag::Delete => {
                line_num += 1;
                // Find the corresponding insert
                let old_line = change.value().trim_end();
                // We'll show the change on this line
                println!(
                    "  {}",
                    format!("- Line {}: {}", line_num, old_line).red()
                );
            }
            ChangeTag::Insert => {
                let new_line = change.value().trim_end();
                println!(
                    "  {}",
                    format!("+ Line {}: {}", line_num, new_line).green()
                );
            }
        }
    }
}

fn print_unified_diff(file: &str, old: &str, new: &str) {
    let diff = TextDiff::from_lines(old, new);

    println!("{}", format!("--- {}", file).red());
    println!("{}", format!("+++ {}", file).green());

    for hunk in diff.unified_diff().context_radius(3).iter_hunks() {
        println!("{}", format!("{}", hunk.header()).cyan());
        for change in hunk.iter_changes() {
            let (sign, color_fn): (&str, fn(&str) -> colored::ColoredString) = match change.tag() {
                ChangeTag::Delete => ("-", |s: &str| s.red()),
                ChangeTag::Insert => ("+", |s: &str| s.green()),
                ChangeTag::Equal => (" ", |s: &str| s.normal()),
            };
            print!("{}", color_fn(&format!("{}{}", sign, change.value())));
        }
    }
}

pub fn lint(
    paths: &[String],
    dialect_name: &str,
    keyword_case: &str,
    no_select_star: bool,
    require_alias: bool,
    messages: &Messages,
) -> Result<()> {
    let dialect = get_dialect(dialect_name)?;
    let files = collect_sql_files(paths);

    if files.is_empty() {
        eprintln!("{}", "No SQL files found".yellow());
        return Ok(());
    }

    let kw_case = match keyword_case.to_lowercase().as_str() {
        "upper" => KeywordCase::Upper,
        "lower" => KeywordCase::Lower,
        "ignore" => KeywordCase::Ignore,
        _ => KeywordCase::Upper,
    };

    let config = LintConfig {
        keyword_case: kw_case,
        no_select_star,
        require_table_alias: require_alias,
        trailing_semicolon: true,
    };

    let linter = Linter::new(config);
    let mut total_warnings = 0;

    for file in &files {
        let content =
            fs::read_to_string(file).with_context(|| format!("Failed to read: {}", file))?;

        let errors = linter.lint(&content, dialect.as_ref(), messages);

        if errors.is_empty() {
            println!("{}", messages.file_ok(file).green());
        } else {
            println!(
                "{}",
                format!("⚠ {} - {} warning(s)", file, errors.len()).yellow()
            );
            for error in &errors {
                println!(
                    "{}",
                    messages.lint_warning(&error.rule, error.line, error.column, &error.message)
                );
            }
            total_warnings += errors.len();
        }
    }

    println!("{}", messages.lint_summary(files.len(), total_warnings));

    if total_warnings > 0 {
        std::process::exit(1);
    }

    Ok(())
}
