use sys_locale::get_locale;

pub fn is_japanese_locale() -> bool {
    get_locale().map(|l| l.starts_with("ja")).unwrap_or(false)
}

pub struct Messages {
    lang: String,
}

impl Messages {
    pub fn new(lang: &str) -> Self {
        Self {
            lang: lang.to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn checking_file(&self, path: &str) -> String {
        match self.lang.as_str() {
            "ja" => format!("チェック中: {}", path),
            _ => format!("Checking: {}", path),
        }
    }

    pub fn syntax_error(&self, line: usize, col: usize, msg: &str) -> String {
        match self.lang.as_str() {
            "ja" => format!("構文エラー ({}行目, {}列目): {}", line, col, msg),
            _ => format!("Syntax error (line {}, col {}): {}", line, col, msg),
        }
    }

    pub fn file_ok(&self, path: &str) -> String {
        match self.lang.as_str() {
            "ja" => format!("✓ {} - 問題なし", path),
            _ => format!("✓ {} - OK", path),
        }
    }

    pub fn file_error(&self, path: &str, count: usize) -> String {
        match self.lang.as_str() {
            "ja" => format!("✗ {} - {}件のエラー", path, count),
            _ => format!("✗ {} - {} error(s)", path, count),
        }
    }

    pub fn summary(&self, files: usize, errors: usize) -> String {
        match self.lang.as_str() {
            "ja" => format!("\n合計: {}ファイル, {}件のエラー", files, errors),
            _ => format!("\nTotal: {} file(s), {} error(s)", files, errors),
        }
    }

    pub fn would_fix(&self, path: &str) -> String {
        match self.lang.as_str() {
            "ja" => format!("修正予定: {}", path),
            _ => format!("Would fix: {}", path),
        }
    }

    pub fn fixed(&self, path: &str) -> String {
        match self.lang.as_str() {
            "ja" => format!("修正完了: {}", path),
            _ => format!("Fixed: {}", path),
        }
    }

    // Lint messages
    pub fn keyword_case_error(&self, actual: &str, expected: &str) -> String {
        match self.lang.as_str() {
            "ja" => format!("キーワード '{}' は '{}' であるべきです", actual, expected),
            _ => format!("Keyword '{}' should be '{}'", actual, expected),
        }
    }

    pub fn no_select_star_error(&self) -> String {
        match self.lang.as_str() {
            "ja" => "SELECT * の使用は推奨されません。カラムを明示的に指定してください".to_string(),
            _ => "Avoid SELECT *. Specify columns explicitly".to_string(),
        }
    }

    pub fn require_table_alias_error(&self, table: &str) -> String {
        match self.lang.as_str() {
            "ja" => format!("テーブル '{}' にはエイリアスを指定してください", table),
            _ => format!("Table '{}' should have an alias", table),
        }
    }

    pub fn trailing_semicolon_error(&self) -> String {
        match self.lang.as_str() {
            "ja" => "文末にセミコロンがありません".to_string(),
            _ => "Missing trailing semicolon".to_string(),
        }
    }

    pub fn lint_warning(&self, rule: &str, line: usize, col: usize, msg: &str) -> String {
        match self.lang.as_str() {
            "ja" => format!("  [{}] {}行目:{}列目 - {}", rule, line, col, msg),
            _ => format!("  [{}] line {}:{} - {}", rule, line, col, msg),
        }
    }

    pub fn lint_summary(&self, files: usize, warnings: usize) -> String {
        match self.lang.as_str() {
            "ja" => format!("\n合計: {}ファイル, {}件の警告", files, warnings),
            _ => format!("\nTotal: {} file(s), {} warning(s)", files, warnings),
        }
    }

    // Hint messages
    pub fn hint_trailing_comma(&self, line: usize) -> String {
        match self.lang.as_str() {
            "ja" => format!("{}行目の末尾に余計なカンマがある可能性があります", line),
            _ => format!(
                "Line {} may have a trailing comma that should be removed",
                line
            ),
        }
    }

    pub fn hint_check_parentheses(&self) -> String {
        match self.lang.as_str() {
            "ja" => "括弧の対応を確認してください".to_string(),
            _ => "Check for mismatched parentheses".to_string(),
        }
    }

    pub fn hint_missing_parentheses(&self) -> String {
        match self.lang.as_str() {
            "ja" => "関数呼び出しに括弧が必要かもしれません".to_string(),
            _ => "Function call may require parentheses".to_string(),
        }
    }

    pub fn hint_unclosed_parentheses(&self, count: i32) -> String {
        match self.lang.as_str() {
            "ja" => format!("閉じ括弧が{}個不足しています", count),
            _ => format!("{} unclosed parenthesis(es) found", count),
        }
    }

    pub fn hint_unclosed_quote(&self) -> String {
        match self.lang.as_str() {
            "ja" => "閉じられていない引用符があります".to_string(),
            _ => "Unclosed quote found".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_japanese_locale_returns_bool() {
        // Just ensure it doesn't panic and returns a value consistent with the env.
        let _ = is_japanese_locale();
    }

    #[test]
    fn test_unknown_lang_falls_back_to_english() {
        // Any lang other than "ja" should produce English output.
        let m = Messages::new("fr");
        assert_eq!(m.file_ok("a.sql"), "✓ a.sql - OK");
        assert_eq!(m.trailing_semicolon_error(), "Missing trailing semicolon");
    }

    #[test]
    fn test_checking_file() {
        assert_eq!(
            Messages::new("en").checking_file("a.sql"),
            "Checking: a.sql"
        );
        assert_eq!(
            Messages::new("ja").checking_file("a.sql"),
            "チェック中: a.sql"
        );
    }

    #[test]
    fn test_syntax_error_both_langs() {
        assert_eq!(
            Messages::new("en").syntax_error(3, 5, "boom"),
            "Syntax error (line 3, col 5): boom"
        );
        assert_eq!(
            Messages::new("ja").syntax_error(3, 5, "boom"),
            "構文エラー (3行目, 5列目): boom"
        );
    }

    #[test]
    fn test_file_status_both_langs() {
        let en = Messages::new("en");
        let ja = Messages::new("ja");
        assert_eq!(en.file_ok("q.sql"), "✓ q.sql - OK");
        assert_eq!(ja.file_ok("q.sql"), "✓ q.sql - 問題なし");
        assert_eq!(en.file_error("q.sql", 2), "✗ q.sql - 2 error(s)");
        assert_eq!(ja.file_error("q.sql", 2), "✗ q.sql - 2件のエラー");
    }

    #[test]
    fn test_summaries_both_langs() {
        let en = Messages::new("en");
        let ja = Messages::new("ja");
        assert_eq!(en.summary(1, 0), "\nTotal: 1 file(s), 0 error(s)");
        assert_eq!(ja.summary(1, 0), "\n合計: 1ファイル, 0件のエラー");
        assert_eq!(en.lint_summary(2, 3), "\nTotal: 2 file(s), 3 warning(s)");
        assert_eq!(ja.lint_summary(2, 3), "\n合計: 2ファイル, 3件の警告");
    }

    #[test]
    fn test_fix_messages_both_langs() {
        let en = Messages::new("en");
        let ja = Messages::new("ja");
        assert_eq!(en.would_fix("q.sql"), "Would fix: q.sql");
        assert_eq!(ja.would_fix("q.sql"), "修正予定: q.sql");
        assert_eq!(en.fixed("q.sql"), "Fixed: q.sql");
        assert_eq!(ja.fixed("q.sql"), "修正完了: q.sql");
    }

    #[test]
    fn test_lint_rule_messages_both_langs() {
        let en = Messages::new("en");
        let ja = Messages::new("ja");
        assert_eq!(
            en.keyword_case_error("select", "SELECT"),
            "Keyword 'select' should be 'SELECT'"
        );
        assert_eq!(
            ja.keyword_case_error("select", "SELECT"),
            "キーワード 'select' は 'SELECT' であるべきです"
        );
        assert_eq!(
            en.no_select_star_error(),
            "Avoid SELECT *. Specify columns explicitly"
        );
        assert!(ja.no_select_star_error().contains("SELECT *"));
        assert_eq!(
            en.require_table_alias_error("users"),
            "Table 'users' should have an alias"
        );
        assert!(ja.require_table_alias_error("users").contains("users"));
    }

    #[test]
    fn test_lint_warning_both_langs() {
        assert_eq!(
            Messages::new("en").lint_warning("keyword-case", 1, 2, "msg"),
            "  [keyword-case] line 1:2 - msg"
        );
        assert_eq!(
            Messages::new("ja").lint_warning("keyword-case", 1, 2, "msg"),
            "  [keyword-case] 1行目:2列目 - msg"
        );
    }

    #[test]
    fn test_hint_messages_both_langs() {
        let en = Messages::new("en");
        let ja = Messages::new("ja");
        assert_eq!(
            en.hint_trailing_comma(4),
            "Line 4 may have a trailing comma that should be removed"
        );
        assert!(ja.hint_trailing_comma(4).contains("4行目"));
        assert_eq!(
            en.hint_check_parentheses(),
            "Check for mismatched parentheses"
        );
        assert_eq!(ja.hint_check_parentheses(), "括弧の対応を確認してください");
        assert_eq!(
            en.hint_missing_parentheses(),
            "Function call may require parentheses"
        );
        assert!(ja.hint_missing_parentheses().contains("括弧"));
        assert_eq!(
            en.hint_unclosed_parentheses(2),
            "2 unclosed parenthesis(es) found"
        );
        assert!(ja.hint_unclosed_parentheses(2).contains("2個"));
        assert_eq!(en.hint_unclosed_quote(), "Unclosed quote found");
        assert_eq!(ja.hint_unclosed_quote(), "閉じられていない引用符があります");
    }
}
