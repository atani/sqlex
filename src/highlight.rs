use colored::Colorize;

pub struct SourceHighlighter;

impl SourceHighlighter {
    /// Display source code with highlighted error location and optional suspect line
    pub fn display_error_with_hint(
        source: &str,
        error_line: usize,
        column: usize,
        suspect_line: Option<usize>,
        context_lines: usize,
    ) -> String {
        let lines: Vec<&str> = source.lines().collect();
        let mut output = Vec::new();

        // Calculate line range to display (expand if suspect line is outside normal range)
        let mut start_line = error_line.saturating_sub(context_lines + 1);
        let mut end_line = (error_line + context_lines).min(lines.len());

        // Expand range to include suspect line if needed
        if let Some(suspect) = suspect_line {
            if suspect < start_line + 1 {
                start_line = suspect.saturating_sub(1);
            }
            if suspect > end_line {
                end_line = suspect.min(lines.len());
            }
        }

        // Line number width for formatting
        let line_num_width = end_line.to_string().len();

        for (idx, line_content) in lines.iter().enumerate() {
            let line_num = idx + 1;
            if line_num < start_line + 1 || line_num > end_line {
                continue;
            }

            let line_num_str = format!("{:>width$}", line_num, width = line_num_width);

            if line_num == error_line {
                // Error line - highlight in red
                output.push(format!(
                    "{} {} {}",
                    line_num_str.red().bold(),
                    "|".red(),
                    line_content
                ));

                // Add caret indicator
                let spaces = " ".repeat(line_num_width);
                let indicator = Self::make_indicator(column, line_content.len());
                output.push(format!(
                    "{} {} {}",
                    spaces,
                    "|".red(),
                    indicator.red().bold()
                ));
            } else if suspect_line == Some(line_num) {
                // Suspect line - highlight in yellow with marker
                output.push(format!(
                    "{} {} {}  {}",
                    line_num_str.yellow().bold(),
                    "|".yellow(),
                    line_content,
                    "← ここを確認".yellow().bold()
                ));
            } else {
                // Context line
                output.push(format!(
                    "{} {} {}",
                    line_num_str.dimmed(),
                    "|".dimmed(),
                    line_content.dimmed()
                ));
            }
        }

        output.join("\n")
    }

    /// Display source code with highlighted error location (backward compatible)
    #[allow(dead_code)]
    pub fn display_error(source: &str, line: usize, column: usize, context_lines: usize) -> String {
        Self::display_error_with_hint(source, line, column, None, context_lines)
    }

    /// Create indicator line with caret pointing to error column
    fn make_indicator(column: usize, line_len: usize) -> String {
        let col = column.saturating_sub(1).min(line_len);
        let mut indicator = " ".repeat(col);
        indicator.push('^');
        indicator
    }

    /// Display multiple lines with optional range highlighting
    #[allow(dead_code)]
    pub fn display_range(
        source: &str,
        start_line: usize,
        end_line: usize,
        start_col: usize,
        end_col: usize,
    ) -> String {
        let lines: Vec<&str> = source.lines().collect();
        let mut output = Vec::new();

        let line_num_width = end_line.to_string().len();

        for (idx, line_content) in lines.iter().enumerate() {
            let line_num = idx + 1;
            if line_num < start_line || line_num > end_line {
                continue;
            }

            let line_num_str = format!("{:>width$}", line_num, width = line_num_width);

            // Highlight the range
            let highlighted = if line_num == start_line && line_num == end_line {
                // Single line range
                Self::highlight_range(line_content, start_col, end_col)
            } else if line_num == start_line {
                Self::highlight_range(line_content, start_col, line_content.len())
            } else if line_num == end_line {
                Self::highlight_range(line_content, 1, end_col)
            } else {
                line_content.yellow().to_string()
            };

            output.push(format!(
                "{} {} {}",
                line_num_str.cyan(),
                "|".dimmed(),
                highlighted
            ));
        }

        output.join("\n")
    }

    fn highlight_range(line: &str, start_col: usize, end_col: usize) -> String {
        let start = start_col.saturating_sub(1);
        let end = end_col.min(line.len());

        let chars: Vec<char> = line.chars().collect();
        let before: String = chars.iter().take(start).collect();
        let highlight: String = chars.iter().skip(start).take(end - start).collect();
        let after: String = chars.iter().skip(end).collect();

        format!("{}{}{}", before, highlight.red().underline(), after)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_error() {
        let source = "SELECT id\nFROM users\nWHERE active =";
        let output = SourceHighlighter::display_error(source, 3, 15, 1);
        assert!(output.contains("WHERE active ="));
    }

    #[test]
    fn test_make_indicator() {
        let indicator = SourceHighlighter::make_indicator(5, 20);
        assert_eq!(indicator, "    ^");
    }

    #[test]
    fn test_make_indicator_clamps_to_line_length() {
        // Column past the end of line is clamped to line length.
        let indicator = SourceHighlighter::make_indicator(100, 3);
        assert_eq!(indicator, "   ^");
    }

    #[test]
    fn test_make_indicator_column_zero() {
        // Column 0/1 points at the first character.
        assert_eq!(SourceHighlighter::make_indicator(1, 10), "^");
    }

    #[test]
    fn test_display_error_renders_caret_and_context() {
        colored::control::set_override(false);
        let source = "SELECT id\nFROM users\nWHERE active =";
        let output = SourceHighlighter::display_error(source, 3, 1, 2);
        // Error line and surrounding context lines are shown.
        assert!(output.contains("SELECT id"));
        assert!(output.contains("FROM users"));
        assert!(output.contains("WHERE active ="));
        // Caret indicator line is present.
        assert!(output.contains("^"));
        // Line numbers are rendered.
        assert!(output.contains("3 |"));
    }

    #[test]
    fn test_display_error_with_suspect_line_below_range() {
        colored::control::set_override(false);
        let source = "a,\nSELECT\nb\nc\nd\ne\nFROM t";
        // Error on line 7, suspect line 1 (outside the default context window).
        let output = SourceHighlighter::display_error_with_hint(source, 7, 1, Some(1), 2);
        // The suspect marker must be rendered.
        assert!(output.contains("← ここを確認"));
        // The suspect line content (line 1) is included even though it's far above.
        assert!(output.contains("a,"));
    }

    #[test]
    fn test_display_error_with_suspect_line_above_range() {
        colored::control::set_override(false);
        let source = "SELECT\na\nb\nc\nd\ne,\nFROM t";
        // Error on line 1, suspect line 6 (below the default context window).
        let output = SourceHighlighter::display_error_with_hint(source, 1, 1, Some(6), 1);
        assert!(output.contains("← ここを確認"));
        assert!(output.contains("e,"));
    }

    #[test]
    fn test_display_range_single_line() {
        colored::control::set_override(false);
        let source = "SELECT abcdef FROM t";
        let output = SourceHighlighter::display_range(source, 1, 1, 8, 13);
        assert!(output.contains("SELECT"));
        assert!(output.contains("abcde"));
    }

    #[test]
    fn test_display_range_multi_line() {
        colored::control::set_override(false);
        let source = "line one\nline two\nline three\nline four";
        let output = SourceHighlighter::display_range(source, 1, 3, 3, 4);
        // Start, middle, and end lines are all rendered.
        assert!(output.contains("line one"));
        assert!(output.contains("line two"));
        assert!(output.contains("line three"));
        // Line four is outside the range.
        assert!(!output.contains("line four"));
    }

    #[test]
    fn test_highlight_range_splits_correctly() {
        colored::control::set_override(false);
        let result = SourceHighlighter::highlight_range("abcdef", 2, 4);
        // before = "a", highlighted = "bcd", after = "ef" → concatenated back to original.
        assert_eq!(result, "abcdef");
    }
}
