use crate::cli::FixFormat;
use crate::error::SqlexError;
use crate::highlight::SourceHighlighter;
use crate::hints;
use crate::i18n::Messages;
use crate::linter::{is_sql_keyword, KeywordCase, LintConfig, Linter};
use anyhow::{Context, Result};
use colored::Colorize;
use similar::{ChangeTag, TextDiff};
use sqlparser::dialect::{
    BigQueryDialect, Dialect, GenericDialect, MySqlDialect, PostgreSqlDialect, SQLiteDialect,
};
use sqlparser::parser::Parser;
use sqlparser::tokenizer::{Token, Tokenizer};
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

        let new_content = fix_content(&content, dialect.as_ref())?;

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

    Ok(())
}

/// Build a mapping from (line, column) to byte offset in the source string.
/// Both line and column are 1-based (matching sqlparser's Location).
fn build_line_offsets(src: &str) -> Vec<usize> {
    let mut offsets = vec![0]; // offsets[0] = byte offset of line 1
    for (i, b) in src.bytes().enumerate() {
        if b == b'\n' {
            offsets.push(i + 1);
        }
    }
    offsets
}

fn location_to_byte_offset(line_offsets: &[usize], line: u64, column: u64) -> usize {
    let line_idx = (line as usize).saturating_sub(1);
    let col_offset = (column as usize).saturating_sub(1);
    if line_idx < line_offsets.len() {
        line_offsets[line_idx] + col_offset
    } else {
        // Fallback: end of string
        line_offsets.last().copied().unwrap_or(0)
    }
}

/// Fix SQL content using token-based partial replacement.
/// Only modifies keyword case and trailing semicolons, preserving all original formatting.
fn fix_content(content: &str, dialect: &dyn Dialect) -> Result<String> {
    let mut result = content.to_string();

    // 1. Fix keyword case using tokenizer (preserves original whitespace/indentation)
    let mut tokenizer = Tokenizer::new(dialect, content);
    match tokenizer.tokenize_with_location() {
        Ok(tokens) => {
            let line_offsets = build_line_offsets(content);

            // Collect replacements: (byte_offset, original_len, replacement)
            let mut replacements: Vec<(usize, usize, String)> = Vec::new();

            for token_with_span in &tokens {
                if let Token::Word(word) = &token_with_span.token {
                    if word.quote_style.is_none() && is_sql_keyword(&word.value) {
                        let upper = word.value.to_uppercase();
                        if word.value != upper {
                            let offset = location_to_byte_offset(
                                &line_offsets,
                                token_with_span.span.start.line,
                                token_with_span.span.start.column,
                            );
                            replacements.push((offset, word.value.len(), upper));
                        }
                    }
                }
            }

            // Apply replacements in reverse order to preserve byte offsets
            for (offset, len, replacement) in replacements.into_iter().rev() {
                if offset + len <= result.len() {
                    result.replace_range(offset..offset + len, &replacement);
                }
            }
        }
        Err(_) => {
            // Tokenization failed; skip keyword fix for this file
        }
    }

    // 2. Fix trailing semicolon
    let trimmed = result.trim_end();
    if !trimmed.is_empty() && !trimmed.ends_with(';') {
        result = trimmed.to_string() + ";\n";
    }

    Ok(result)
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
                println!("  {}", format!("- Line {}: {}", line_num, old_line).red());
            }
            ChangeTag::Insert => {
                let new_line = change.value().trim_end();
                println!("  {}", format!("+ Line {}: {}", line_num, new_line).green());
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_get_dialect_known() {
        for name in [
            "generic",
            "mysql",
            "postgres",
            "postgresql",
            "sqlite",
            "bigquery",
        ] {
            assert!(
                get_dialect(name).is_ok(),
                "dialect {name} should be supported"
            );
        }
        // Case-insensitive.
        assert!(get_dialect("MySQL").is_ok());
    }

    #[test]
    fn test_get_dialect_unknown_errors() {
        let err = get_dialect("oracle").unwrap_err();
        assert!(err.to_string().contains("Unsupported dialect"));
        assert!(err.to_string().contains("oracle"));
    }

    #[test]
    fn test_parse_error_location_colon_format() {
        let (line, col) = parse_error_location("sql parser error at Line: 5, Column: 6");
        assert_eq!((line, col), (5, 6));
    }

    #[test]
    fn test_parse_error_location_space_format() {
        let (line, col) = parse_error_location("error at Line: 12, Column 3");
        assert_eq!((line, col), (12, 3));
    }

    #[test]
    fn test_parse_error_location_defaults_when_absent() {
        let (line, col) = parse_error_location("some error without location");
        assert_eq!((line, col), (1, 1));
    }

    #[test]
    fn test_check_sql_valid_returns_no_errors() {
        let d = GenericDialect {};
        assert!(check_sql("SELECT id FROM users;", &d).is_empty());
    }

    #[test]
    fn test_check_sql_invalid_returns_error() {
        let d = GenericDialect {};
        let errors = check_sql("SELECT id FROM users WHERE;", &d);
        assert_eq!(errors.len(), 1);
        assert!(!errors[0].message.is_empty());
    }

    #[test]
    fn test_build_line_offsets() {
        // "a\nbb\nccc" → line starts at byte 0, 2, 5.
        let offsets = build_line_offsets("a\nbb\nccc");
        assert_eq!(offsets, vec![0, 2, 5]);
    }

    #[test]
    fn test_build_line_offsets_single_line() {
        assert_eq!(build_line_offsets("SELECT 1"), vec![0]);
    }

    #[test]
    fn test_location_to_byte_offset() {
        let offsets = build_line_offsets("abc\ndef");
        // Line 2, column 1 → byte offset 4 (start of "def").
        assert_eq!(location_to_byte_offset(&offsets, 2, 1), 4);
        // Line 1, column 3 → byte offset 2.
        assert_eq!(location_to_byte_offset(&offsets, 1, 3), 2);
    }

    #[test]
    fn test_location_to_byte_offset_out_of_range_falls_back() {
        let offsets = build_line_offsets("abc");
        // Line beyond the input → falls back to last known offset.
        assert_eq!(location_to_byte_offset(&offsets, 99, 1), 0);
    }

    #[test]
    fn test_fix_content_uppercases_keywords() {
        let d = GenericDialect {};
        let out = fix_content("select id from users;", &d).unwrap();
        assert!(out.contains("SELECT"));
        assert!(out.contains("FROM"));
        // Identifiers are preserved as-is.
        assert!(out.contains("id"));
        assert!(out.contains("users"));
    }

    #[test]
    fn test_fix_content_adds_trailing_semicolon() {
        let d = GenericDialect {};
        let out = fix_content("SELECT 1", &d).unwrap();
        assert!(out.trim_end().ends_with(';'));
    }

    #[test]
    fn test_fix_content_preserves_existing_semicolon() {
        let d = GenericDialect {};
        let out = fix_content("SELECT 1;\n", &d).unwrap();
        // Already terminated → no extra semicolon appended.
        assert_eq!(out.matches(';').count(), 1);
    }

    #[test]
    fn test_fix_content_preserves_whitespace_and_quoted_identifiers() {
        let d = GenericDialect {};
        // Double whitespace between tokens must be preserved (token-based replacement).
        let out = fix_content("select  id  from  users;", &d).unwrap();
        assert!(out.contains("SELECT  id  FROM  users"));
    }

    #[test]
    fn test_collect_sql_files_filters_extension() {
        let dir = TempDir::new().unwrap();
        let sql = dir.path().join("a.sql");
        let txt = dir.path().join("b.txt");
        fs::write(&sql, "SELECT 1;").unwrap();
        fs::write(&txt, "not sql").unwrap();

        // Directory traversal picks up only the .sql file.
        let files = collect_sql_files(&[dir.path().to_string_lossy().to_string()]);
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("a.sql"));
    }

    #[test]
    fn test_collect_sql_files_single_file() {
        let dir = TempDir::new().unwrap();
        let sql = dir.path().join("q.sql");
        fs::write(&sql, "SELECT 1;").unwrap();
        let files = collect_sql_files(&[sql.to_string_lossy().to_string()]);
        assert_eq!(files.len(), 1);
    }

    #[test]
    fn test_collect_sql_files_ignores_non_sql_single_file() {
        let dir = TempDir::new().unwrap();
        let txt = dir.path().join("q.txt");
        fs::write(&txt, "nope").unwrap();
        let files = collect_sql_files(&[txt.to_string_lossy().to_string()]);
        assert!(files.is_empty());
    }

    #[test]
    fn test_dialect_specific_parsing() {
        // BigQuery accepts backtick-quoted identifiers.
        let bq = get_dialect("bigquery").unwrap();
        assert!(check_sql("SELECT `col` FROM `t`;", bq.as_ref()).is_empty());

        // PostgreSQL accepts double-quoted identifiers.
        let pg = get_dialect("postgres").unwrap();
        assert!(check_sql("SELECT \"col\" FROM \"t\";", pg.as_ref()).is_empty());
    }
}
