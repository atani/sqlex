use colored::Colorize;

pub struct SourceHighlighter;

impl SourceHighlighter {
    /// Display source code with highlighted error location
    pub fn display_error(source: &str, line: usize, column: usize, context_lines: usize) -> String {
        let lines: Vec<&str> = source.lines().collect();
        let mut output = Vec::new();

        // Calculate line range to display
        let start_line = line.saturating_sub(context_lines + 1);
        let end_line = (line + context_lines).min(lines.len());

        // Line number width for formatting
        let line_num_width = end_line.to_string().len();

        for (idx, line_content) in lines.iter().enumerate() {
            let line_num = idx + 1;
            if line_num < start_line + 1 || line_num > end_line {
                continue;
            }

            let line_num_str = format!("{:>width$}", line_num, width = line_num_width);

            if line_num == line {
                // Error line - highlight
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
}
