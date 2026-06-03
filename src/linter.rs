use crate::i18n::Messages;
use sqlparser::ast::{SelectItem, SetExpr, Statement, TableFactor, TableWithJoins};
use sqlparser::dialect::Dialect;
use sqlparser::parser::Parser;
use sqlparser::tokenizer::{Token, Tokenizer};

#[derive(Debug, Clone)]
pub struct LintError {
    pub rule: String,
    pub line: usize,
    pub column: usize,
    pub message: String,
    #[allow(dead_code)]
    pub severity: Severity,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct LintConfig {
    pub keyword_case: KeywordCase,
    pub no_select_star: bool,
    pub require_table_alias: bool,
    pub trailing_semicolon: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeywordCase {
    Upper,
    Lower,
    Ignore,
}

impl Default for LintConfig {
    fn default() -> Self {
        Self {
            keyword_case: KeywordCase::Upper,
            no_select_star: true,
            require_table_alias: false,
            trailing_semicolon: true,
        }
    }
}

pub struct Linter {
    config: LintConfig,
}

impl Linter {
    pub fn new(config: LintConfig) -> Self {
        Self { config }
    }

    pub fn lint(&self, sql: &str, dialect: &dyn Dialect, messages: &Messages) -> Vec<LintError> {
        let mut errors = Vec::new();

        // Keyword case check using tokenizer
        if self.config.keyword_case != KeywordCase::Ignore {
            errors.extend(self.check_keyword_case(sql, dialect, messages));
        }

        // AST-based checks
        if let Ok(statements) = Parser::parse_sql(dialect, sql) {
            for stmt in &statements {
                if self.config.no_select_star {
                    errors.extend(self.check_select_star(stmt, messages));
                }
                if self.config.require_table_alias {
                    errors.extend(self.check_table_alias(stmt, messages));
                }
            }
        }

        // Trailing semicolon check
        if self.config.trailing_semicolon {
            errors.extend(self.check_trailing_semicolon(sql, messages));
        }

        errors
    }

    fn check_keyword_case(
        &self,
        sql: &str,
        dialect: &dyn Dialect,
        messages: &Messages,
    ) -> Vec<LintError> {
        let mut errors = Vec::new();
        let mut tokenizer = Tokenizer::new(dialect, sql);

        // Tokenize with location so each keyword carries an exact (line, column),
        // matching the fix command and avoiding the previous approximate scan.
        if let Ok(tokens) = tokenizer.tokenize_with_location() {
            for token_with_span in tokens {
                let Token::Word(word) = &token_with_span.token else {
                    continue;
                };
                // Quoted identifiers that happen to match a keyword are not keywords.
                if word.quote_style.is_some() || !is_sql_keyword(&word.value) {
                    continue;
                }

                // This method is only invoked for Upper/Lower (the caller skips
                // Ignore), so treat anything that is not Upper as Lower.
                let want_upper = self.config.keyword_case == KeywordCase::Upper;
                let conforms = if want_upper {
                    word.value.chars().all(|c| c.is_uppercase())
                } else {
                    word.value.chars().all(|c| c.is_lowercase())
                };
                if conforms {
                    continue;
                }

                let expected = if want_upper {
                    word.value.to_uppercase()
                } else {
                    word.value.to_lowercase()
                };
                let start = token_with_span.span.start;
                errors.push(LintError {
                    rule: "keyword-case".to_string(),
                    line: (start.line as usize).max(1),
                    column: (start.column as usize).max(1),
                    message: messages.keyword_case_error(&word.value, &expected),
                    severity: Severity::Warning,
                });
            }
        }

        errors
    }

    fn check_select_star(&self, stmt: &Statement, messages: &Messages) -> Vec<LintError> {
        let mut errors = Vec::new();

        if let Statement::Query(query) = stmt {
            if let SetExpr::Select(select) = query.body.as_ref() {
                for item in &select.projection {
                    if matches!(item, SelectItem::Wildcard(_)) {
                        errors.push(LintError {
                            rule: "no-select-star".to_string(),
                            line: 1,
                            column: 1,
                            message: messages.no_select_star_error(),
                            severity: Severity::Warning,
                        });
                    }
                    // Check for table.* pattern
                    if let SelectItem::QualifiedWildcard(_, _) = item {
                        errors.push(LintError {
                            rule: "no-select-star".to_string(),
                            line: 1,
                            column: 1,
                            message: messages.no_select_star_error(),
                            severity: Severity::Warning,
                        });
                    }
                }
            }
        }

        errors
    }

    fn check_table_alias(&self, stmt: &Statement, messages: &Messages) -> Vec<LintError> {
        let mut errors = Vec::new();

        if let Statement::Query(query) = stmt {
            if let SetExpr::Select(select) = query.body.as_ref() {
                for table in &select.from {
                    self.check_table_with_joins(table, &mut errors, messages);
                }
            }
        }

        errors
    }

    fn check_table_with_joins(
        &self,
        table: &TableWithJoins,
        errors: &mut Vec<LintError>,
        messages: &Messages,
    ) {
        if let TableFactor::Table { name, alias, .. } = &table.relation {
            if alias.is_none() {
                errors.push(LintError {
                    rule: "require-table-alias".to_string(),
                    line: 1,
                    column: 1,
                    message: messages.require_table_alias_error(&name.to_string()),
                    severity: Severity::Warning,
                });
            }
        }

        for join in &table.joins {
            if let TableFactor::Table { name, alias, .. } = &join.relation {
                if alias.is_none() {
                    errors.push(LintError {
                        rule: "require-table-alias".to_string(),
                        line: 1,
                        column: 1,
                        message: messages.require_table_alias_error(&name.to_string()),
                        severity: Severity::Warning,
                    });
                }
            }
        }
    }

    fn check_trailing_semicolon(&self, sql: &str, messages: &Messages) -> Vec<LintError> {
        let trimmed = sql.trim();
        if !trimmed.is_empty() && !trimmed.ends_with(';') {
            let lines: Vec<&str> = sql.lines().collect();
            vec![LintError {
                rule: "trailing-semicolon".to_string(),
                line: lines.len(),
                column: lines.last().map(|l| l.len()).unwrap_or(1),
                message: messages.trailing_semicolon_error(),
                severity: Severity::Warning,
            }]
        } else {
            vec![]
        }
    }
}

pub fn is_sql_keyword(word: &str) -> bool {
    const KEYWORDS: &[&str] = &[
        "SELECT",
        "FROM",
        "WHERE",
        "AND",
        "OR",
        "NOT",
        "IN",
        "IS",
        "NULL",
        "LIKE",
        "BETWEEN",
        "JOIN",
        "LEFT",
        "RIGHT",
        "INNER",
        "OUTER",
        "FULL",
        "CROSS",
        "ON",
        "AS",
        "ORDER",
        "BY",
        "ASC",
        "DESC",
        "GROUP",
        "HAVING",
        "LIMIT",
        "OFFSET",
        "UNION",
        "ALL",
        "DISTINCT",
        "INSERT",
        "INTO",
        "VALUES",
        "UPDATE",
        "SET",
        "DELETE",
        "CREATE",
        "TABLE",
        "INDEX",
        "VIEW",
        "DROP",
        "ALTER",
        "ADD",
        "COLUMN",
        "PRIMARY",
        "KEY",
        "FOREIGN",
        "REFERENCES",
        "CONSTRAINT",
        "DEFAULT",
        "UNIQUE",
        "CHECK",
        "CASCADE",
        "RESTRICT",
        "IF",
        "EXISTS",
        "CASE",
        "WHEN",
        "THEN",
        "ELSE",
        "END",
        "CAST",
        "COALESCE",
        "NULLIF",
        "TRUE",
        "FALSE",
        "WITH",
        "RECURSIVE",
        "WINDOW",
        "OVER",
        "PARTITION",
        "ROWS",
        "RANGE",
        "UNBOUNDED",
        "PRECEDING",
        "FOLLOWING",
        "CURRENT",
        "ROW",
        "EXCEPT",
        "INTERSECT",
        "FETCH",
        "FIRST",
        "NEXT",
        "ONLY",
        "PERCENT",
        "TIES",
        "FOR",
        "SHARE",
        "NOWAIT",
        "SKIP",
        "LOCKED",
    ];
    KEYWORDS.contains(&word.to_uppercase().as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::Messages;
    use sqlparser::dialect::GenericDialect;

    #[test]
    fn test_keyword_case_upper() {
        let linter = Linter::new(LintConfig {
            keyword_case: KeywordCase::Upper,
            ..Default::default()
        });
        let messages = Messages::new("en");
        let dialect = GenericDialect {};

        let errors = linter.lint("select * from users;", &dialect, &messages);
        assert!(errors.iter().any(|e| e.rule == "keyword-case"));
    }

    #[test]
    fn test_no_select_star() {
        let linter = Linter::new(LintConfig {
            no_select_star: true,
            keyword_case: KeywordCase::Ignore,
            ..Default::default()
        });
        let messages = Messages::new("en");
        let dialect = GenericDialect {};

        let errors = linter.lint("SELECT * FROM users;", &dialect, &messages);
        assert!(errors.iter().any(|e| e.rule == "no-select-star"));

        let errors = linter.lint("SELECT id, name FROM users;", &dialect, &messages);
        assert!(!errors.iter().any(|e| e.rule == "no-select-star"));
    }

    #[test]
    fn test_trailing_semicolon() {
        let linter = Linter::new(LintConfig {
            trailing_semicolon: true,
            keyword_case: KeywordCase::Ignore,
            no_select_star: false,
            ..Default::default()
        });
        let messages = Messages::new("en");
        let dialect = GenericDialect {};

        let errors = linter.lint("SELECT * FROM users", &dialect, &messages);
        assert!(errors.iter().any(|e| e.rule == "trailing-semicolon"));

        let errors = linter.lint("SELECT * FROM users;", &dialect, &messages);
        assert!(!errors.iter().any(|e| e.rule == "trailing-semicolon"));
    }

    fn upper_only_linter() -> Linter {
        Linter::new(LintConfig {
            keyword_case: KeywordCase::Upper,
            no_select_star: false,
            require_table_alias: false,
            trailing_semicolon: false,
        })
    }

    #[test]
    fn test_keyword_case_lower_flags_uppercase() {
        let linter = Linter::new(LintConfig {
            keyword_case: KeywordCase::Lower,
            no_select_star: false,
            require_table_alias: false,
            trailing_semicolon: false,
        });
        let messages = Messages::new("en");
        let dialect = GenericDialect {};

        // Uppercase keywords are violations when lower is required.
        let errors = linter.lint("SELECT id FROM users;", &dialect, &messages);
        assert!(errors.iter().any(|e| e.rule == "keyword-case"));

        // Already-lowercase keywords pass.
        let errors = linter.lint("select id from users;", &dialect, &messages);
        assert!(!errors.iter().any(|e| e.rule == "keyword-case"));
    }

    #[test]
    fn test_keyword_case_upper_passes_when_uppercase() {
        let linter = upper_only_linter();
        let messages = Messages::new("en");
        let dialect = GenericDialect {};
        let errors = linter.lint("SELECT id FROM users", &dialect, &messages);
        assert!(!errors.iter().any(|e| e.rule == "keyword-case"));
    }

    #[test]
    fn test_keyword_case_ignore_emits_nothing() {
        let linter = Linter::new(LintConfig {
            keyword_case: KeywordCase::Ignore,
            no_select_star: false,
            require_table_alias: false,
            trailing_semicolon: false,
        });
        let messages = Messages::new("en");
        let dialect = GenericDialect {};
        let errors = linter.lint("select ID from Users", &dialect, &messages);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_keyword_case_error_message_carries_expected() {
        let linter = upper_only_linter();
        let messages = Messages::new("en");
        let dialect = GenericDialect {};
        let errors = linter.lint("select 1", &dialect, &messages);
        let e = errors.iter().find(|e| e.rule == "keyword-case").unwrap();
        assert!(e.message.contains("SELECT"));
        assert_eq!(e.severity, Severity::Warning);
    }

    #[test]
    fn test_require_table_alias_flags_missing_alias() {
        let linter = Linter::new(LintConfig {
            keyword_case: KeywordCase::Ignore,
            no_select_star: false,
            require_table_alias: true,
            trailing_semicolon: false,
        });
        let messages = Messages::new("en");
        let dialect = GenericDialect {};

        // No alias → warning.
        let errors = linter.lint("SELECT a FROM users;", &dialect, &messages);
        assert!(errors.iter().any(|e| e.rule == "require-table-alias"));

        // With alias → no warning.
        let errors = linter.lint("SELECT a FROM users u;", &dialect, &messages);
        assert!(!errors.iter().any(|e| e.rule == "require-table-alias"));
    }

    #[test]
    fn test_require_table_alias_checks_joins() {
        let linter = Linter::new(LintConfig {
            keyword_case: KeywordCase::Ignore,
            no_select_star: false,
            require_table_alias: true,
            trailing_semicolon: false,
        });
        let messages = Messages::new("en");
        let dialect = GenericDialect {};

        // Base table aliased, joined table not → exactly one warning.
        let errors = linter.lint(
            "SELECT a FROM users u JOIN orders ON u.id = orders.user_id;",
            &dialect,
            &messages,
        );
        let alias_errors: Vec<_> = errors
            .iter()
            .filter(|e| e.rule == "require-table-alias")
            .collect();
        assert_eq!(alias_errors.len(), 1);
        assert!(alias_errors[0].message.contains("orders"));
    }

    #[test]
    fn test_qualified_wildcard_flagged() {
        let linter = Linter::new(LintConfig {
            keyword_case: KeywordCase::Ignore,
            no_select_star: true,
            require_table_alias: false,
            trailing_semicolon: false,
        });
        let messages = Messages::new("en");
        let dialect = GenericDialect {};

        // `users.*` is a qualified wildcard and must be flagged.
        let errors = linter.lint("SELECT users.* FROM users;", &dialect, &messages);
        assert!(errors.iter().any(|e| e.rule == "no-select-star"));
    }

    #[test]
    fn test_empty_input_produces_no_keyword_or_star_errors() {
        let linter = Linter::new(LintConfig::default());
        let messages = Messages::new("en");
        let dialect = GenericDialect {};
        // Empty trimmed input → no trailing-semicolon warning either.
        let errors = linter.lint("   \n  ", &dialect, &messages);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_multiple_rules_combined() {
        // Default config: upper keywords, no select star, trailing semicolon.
        let linter = Linter::new(LintConfig::default());
        let messages = Messages::new("en");
        let dialect = GenericDialect {};
        let errors = linter.lint("select * from users", &dialect, &messages);
        assert!(errors.iter().any(|e| e.rule == "keyword-case"));
        assert!(errors.iter().any(|e| e.rule == "no-select-star"));
        assert!(errors.iter().any(|e| e.rule == "trailing-semicolon"));
    }

    #[test]
    fn test_keyword_case_reports_accurate_line_and_column() {
        let linter = upper_only_linter();
        let messages = Messages::new("en");
        let dialect = GenericDialect {};
        // Keyword "from" is on line 2, column 1 (token spans are exact now).
        let errors = linter.lint("SELECT id\nfrom users", &dialect, &messages);
        let kw = errors.iter().find(|e| e.rule == "keyword-case").unwrap();
        assert_eq!((kw.line, kw.column), (2, 1));
    }

    #[test]
    fn test_keyword_case_ignores_quoted_identifiers() {
        let linter = upper_only_linter();
        let messages = Messages::new("en");
        let dialect = GenericDialect {};
        // A double-quoted identifier matching a keyword must not be flagged.
        let errors = linter.lint("SELECT \"select\" FROM t", &dialect, &messages);
        assert!(!errors.iter().any(|e| e.rule == "keyword-case"));
    }

    #[test]
    fn test_is_sql_keyword() {
        assert!(is_sql_keyword("select"));
        assert!(is_sql_keyword("SELECT"));
        assert!(is_sql_keyword("Join"));
        assert!(!is_sql_keyword("users"));
        assert!(!is_sql_keyword("foobar"));
    }

    #[test]
    fn test_lint_config_default() {
        let cfg = LintConfig::default();
        assert_eq!(cfg.keyword_case, KeywordCase::Upper);
        assert!(cfg.no_select_star);
        assert!(!cfg.require_table_alias);
        assert!(cfg.trailing_semicolon);
    }
}
