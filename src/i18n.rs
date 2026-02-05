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
            _ => format!("Line {} may have a trailing comma that should be removed", line),
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
