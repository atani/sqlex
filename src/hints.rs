use crate::i18n::Messages;

/// Analyze error message and source to provide helpful hints
pub struct ErrorHint {
    pub hint: String,
    pub suspect_line: Option<usize>,
    #[allow(dead_code)]
    pub suspect_pattern: Option<String>,
}

/// SQL keywords that typically start a new clause
const CLAUSE_KEYWORDS: &[&str] = &[
    "SELECT", "FROM", "WHERE", "JOIN", "LEFT", "RIGHT", "INNER", "OUTER", "FULL", "CROSS",
    "ON", "AND", "OR", "ORDER", "GROUP", "HAVING", "LIMIT", "OFFSET", "UNION", "INSERT",
    "UPDATE", "DELETE", "SET", "VALUES", "INTO",
];

pub fn analyze_error(
    error_msg: &str,
    source: &str,
    error_line: usize,
    messages: &Messages,
) -> Option<ErrorHint> {
    let lines: Vec<&str> = source.lines().collect();

    // Pattern 1: "Expected: ..., found: ..."
    // → Likely trailing comma before keyword
    if error_msg.contains("Expected:") && error_msg.contains("found:") {
        // Look for trailing comma in the lines before the error
        // Strategy: Find the nearest keyword line above error, then check if the line before it has comma

        if error_line > 1 && error_line <= lines.len() {
            // First, find the nearest keyword line at or before error line
            let mut keyword_line: Option<usize> = None;

            for check_idx in (0..error_line).rev() {
                let line_content = lines[check_idx].trim().to_uppercase();
                if CLAUSE_KEYWORDS.iter().any(|kw| line_content.starts_with(kw)) {
                    keyword_line = Some(check_idx + 1); // 1-indexed
                    break;
                }
            }

            // If we found a keyword line, check if the line before it ends with comma
            if let Some(kw_line) = keyword_line {
                if kw_line > 1 {
                    let prev_line_idx = kw_line - 2; // 0-indexed, line before keyword
                    let prev_line = lines[prev_line_idx].trim();

                    if prev_line.ends_with(',') {
                        return Some(ErrorHint {
                            hint: messages.hint_trailing_comma(kw_line - 1),
                            suspect_line: Some(kw_line - 1),
                            suspect_pattern: Some(",".to_string()),
                        });
                    }
                }
            }

            // Also check if the error line itself is after a comma-ending line
            // (for cases where error is on the keyword line)
            if error_line > 1 {
                let prev_line = lines[error_line - 2].trim();
                if prev_line.ends_with(',') {
                    return Some(ErrorHint {
                        hint: messages.hint_trailing_comma(error_line - 1),
                        suspect_line: Some(error_line - 1),
                        suspect_pattern: Some(",".to_string()),
                    });
                }
            }
        }
    }

    // Pattern 2: "Expected: ), found: ," or similar
    // → Likely mismatched parentheses or extra comma
    if error_msg.contains("Expected: )") {
        return Some(ErrorHint {
            hint: messages.hint_check_parentheses(),
            suspect_line: None,
            suspect_pattern: None,
        });
    }

    // Pattern 3: "Expected: (, found: identifier"
    // → Missing parentheses for function call
    if error_msg.contains("Expected: (") {
        return Some(ErrorHint {
            hint: messages.hint_missing_parentheses(),
            suspect_line: None,
            suspect_pattern: None,
        });
    }

    // Pattern 4: Unexpected end of input
    if error_msg.contains("EOF") || error_msg.contains("end of") {
        // Check for unclosed quotes or parentheses
        let open_parens: i32 = source.chars().filter(|&c| c == '(').count() as i32;
        let close_parens: i32 = source.chars().filter(|&c| c == ')').count() as i32;

        if open_parens > close_parens {
            return Some(ErrorHint {
                hint: messages.hint_unclosed_parentheses(open_parens - close_parens),
                suspect_line: None,
                suspect_pattern: Some("(".to_string()),
            });
        }

        // Check for unclosed quotes
        let single_quotes = source.chars().filter(|&c| c == '\'').count();
        if single_quotes % 2 != 0 {
            return Some(ErrorHint {
                hint: messages.hint_unclosed_quote(),
                suspect_line: None,
                suspect_pattern: Some("'".to_string()),
            });
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trailing_comma_detection() {
        let source = r#"SELECT
  id,
  name,
  email,
WHERE
  active = 1"#;
        let messages = Messages::new("en");
        let hint = analyze_error("Expected: =, found: active", source, 6, &messages);

        assert!(hint.is_some());
        let hint = hint.unwrap();
        assert!(hint.suspect_line.is_some());
        assert_eq!(hint.suspect_line.unwrap(), 4); // Line with trailing comma
    }

    #[test]
    fn test_trailing_comma_detection_long() {
        let source = r#"SELECT
  id,
  name,
  department,
WHERE
  bill_id = 'test'"#;
        let messages = Messages::new("en");
        let hint = analyze_error(
            "Expected: end of statement, found: =",
            source,
            6,
            &messages,
        );

        assert!(hint.is_some());
        let hint = hint.unwrap();
        assert_eq!(hint.suspect_line, Some(4)); // Line with trailing comma before WHERE
    }
}
