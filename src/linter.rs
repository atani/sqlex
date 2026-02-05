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

        if let Ok(tokens) = tokenizer.tokenize() {
            let mut pos = 0;

            for token in tokens {
                let token_str = token.to_string();
                let token_len = token_str.len();

                // Calculate line and column
                let (line, column) = self.pos_to_line_col(sql, pos);

                if let Token::Word(word) = &token {
                    if is_sql_keyword(&word.value) {
                        let is_upper = word.value.chars().all(|c| c.is_uppercase());
                        let is_lower = word.value.chars().all(|c| c.is_lowercase());

                        let violation = match self.config.keyword_case {
                            KeywordCase::Upper => !is_upper,
                            KeywordCase::Lower => !is_lower,
                            KeywordCase::Ignore => false,
                        };

                        if violation {
                            let expected = match self.config.keyword_case {
                                KeywordCase::Upper => word.value.to_uppercase(),
                                KeywordCase::Lower => word.value.to_lowercase(),
                                KeywordCase::Ignore => word.value.clone(),
                            };
                            errors.push(LintError {
                                rule: "keyword-case".to_string(),
                                line,
                                column,
                                message: messages.keyword_case_error(&word.value, &expected),
                                severity: Severity::Warning,
                            });
                        }
                    }
                }

                // Advance position (approximate - tokens may have whitespace between)
                if let Some(idx) = sql[pos..].find(&token_str) {
                    pos += idx + token_len;
                }
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

    fn pos_to_line_col(&self, sql: &str, pos: usize) -> (usize, usize) {
        let mut line = 1;
        let mut col = 1;
        for (i, c) in sql.chars().enumerate() {
            if i >= pos {
                break;
            }
            if c == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        (line, col)
    }
}

fn is_sql_keyword(word: &str) -> bool {
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
}
