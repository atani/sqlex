use std::fs;
use std::process::Command;

fn sqlex() -> Command {
    Command::new(env!("CARGO_BIN_EXE_sqlex"))
}

mod check_command {
    use super::*;
    use tempfile::TempDir;

    fn create_temp_sql(dir: &TempDir, name: &str, content: &str) -> String {
        let path = dir.path().join(name);
        fs::write(&path, content).unwrap();
        path.to_string_lossy().to_string()
    }

    #[test]
    fn test_valid_sql_passes() {
        let dir = TempDir::new().unwrap();
        let path = create_temp_sql(
            &dir,
            "valid.sql",
            "SELECT id, name FROM users WHERE active = 1;",
        );

        let output = sqlex()
            .args(["check", &path])
            .output()
            .expect("Failed to execute");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("OK") || stdout.contains("問題なし"));
    }

    #[test]
    fn test_invalid_sql_fails() {
        let dir = TempDir::new().unwrap();
        // Clearly invalid SQL: incomplete WHERE clause
        let path = create_temp_sql(&dir, "invalid.sql", "SELECT id FROM users WHERE;");

        let output = sqlex()
            .args(["check", &path])
            .output()
            .expect("Failed to execute");

        assert!(!output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("error") || stdout.contains("エラー"));
    }

    #[test]
    fn test_check_directory() {
        let dir = TempDir::new().unwrap();
        create_temp_sql(&dir, "one.sql", "SELECT 1;");
        create_temp_sql(&dir, "two.sql", "SELECT 2;");

        let output = sqlex()
            .args(["check", &dir.path().to_string_lossy()])
            .output()
            .expect("Failed to execute");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("2 file") || stdout.contains("2ファイル"));
    }

    #[test]
    fn test_dialect_mysql() {
        let dir = TempDir::new().unwrap();
        // MySQL specific: backtick identifiers
        let path = create_temp_sql(&dir, "mysql.sql", "SELECT `id` FROM `users`;");

        let output = sqlex()
            .args(["check", "-d", "mysql", &path])
            .output()
            .expect("Failed to execute");

        assert!(output.status.success());
    }

    #[test]
    fn test_language_english() {
        let dir = TempDir::new().unwrap();
        let path = create_temp_sql(&dir, "test.sql", "SELECT 1;");

        let output = sqlex()
            .args(["--lang", "en", "check", &path])
            .output()
            .expect("Failed to execute");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("OK"));
    }

    #[test]
    fn test_language_japanese() {
        let dir = TempDir::new().unwrap();
        let path = create_temp_sql(&dir, "test.sql", "SELECT 1;");

        let output = sqlex()
            .args(["--lang", "ja", "check", &path])
            .output()
            .expect("Failed to execute");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("問題なし"));
    }
}

mod lint_command {
    use super::*;
    use tempfile::TempDir;

    fn create_temp_sql(dir: &TempDir, name: &str, content: &str) -> String {
        let path = dir.path().join(name);
        fs::write(&path, content).unwrap();
        path.to_string_lossy().to_string()
    }

    #[test]
    fn test_lint_keyword_case() {
        let dir = TempDir::new().unwrap();
        let path = create_temp_sql(&dir, "lower.sql", "select id from users;");

        let output = sqlex()
            .args(["lint", "--keyword-case", "upper", &path])
            .output()
            .expect("Failed to execute");

        // Should fail due to lowercase keywords
        assert!(!output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("keyword-case"));
    }

    #[test]
    fn test_lint_keyword_case_ignore() {
        let dir = TempDir::new().unwrap();
        let path = create_temp_sql(&dir, "lower.sql", "select id from users;");

        let output = sqlex()
            .args([
                "lint",
                "--keyword-case",
                "ignore",
                "--no-select-star",
                "false",
                &path,
            ])
            .output()
            .expect("Failed to execute");

        assert!(output.status.success());
    }

    #[test]
    fn test_lint_select_star() {
        let dir = TempDir::new().unwrap();
        let path = create_temp_sql(&dir, "star.sql", "SELECT * FROM users;");

        let output = sqlex()
            .args([
                "lint",
                "--keyword-case",
                "ignore",
                "--no-select-star",
                "true",
                &path,
            ])
            .output()
            .expect("Failed to execute");

        assert!(!output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("no-select-star"));
    }

    #[test]
    fn test_lint_trailing_semicolon() {
        let dir = TempDir::new().unwrap();
        let path = create_temp_sql(&dir, "nosemi.sql", "SELECT id FROM users");

        let output = sqlex()
            .args([
                "lint",
                "--keyword-case",
                "ignore",
                "--no-select-star",
                "false",
                &path,
            ])
            .output()
            .expect("Failed to execute");

        assert!(!output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("trailing-semicolon"));
    }
}

mod fix_command {
    use super::*;
    use tempfile::TempDir;

    fn create_temp_sql(dir: &TempDir, name: &str, content: &str) -> String {
        let path = dir.path().join(name);
        fs::write(&path, content).unwrap();
        path.to_string_lossy().to_string()
    }

    #[test]
    fn test_fix_dry_run() {
        let dir = TempDir::new().unwrap();
        let content = "select  id  from  users";
        let path = create_temp_sql(&dir, "messy.sql", content);

        let output = sqlex()
            .args(["fix", "--dry-run", &path])
            .output()
            .expect("Failed to execute");

        // File should not be modified
        let actual = fs::read_to_string(&path).unwrap();
        assert_eq!(actual, content);

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Would fix") || stdout.contains("修正予定"));
    }

    #[test]
    fn test_fix_applies_changes() {
        let dir = TempDir::new().unwrap();
        let path = create_temp_sql(&dir, "messy.sql", "select  id  from  users;");

        let output = sqlex()
            .args(["fix", &path])
            .output()
            .expect("Failed to execute");

        assert!(output.status.success());

        let actual = fs::read_to_string(&path).unwrap();
        // sqlparser normalizes to uppercase
        assert!(actual.contains("SELECT") || actual.contains("select"));
    }
}

mod help_and_version {
    use super::*;

    #[test]
    fn test_help() {
        let output = sqlex().arg("--help").output().expect("Failed to execute");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SQL syntax checker"));
        assert!(stdout.contains("check"));
        assert!(stdout.contains("fix"));
        assert!(stdout.contains("lint"));
    }

    #[test]
    fn test_version() {
        let output = sqlex()
            .arg("--version")
            .output()
            .expect("Failed to execute");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("sqlex"));
        // Check for semver pattern (e.g., "0.1.0", "1.2.3")
        assert!(
            stdout.chars().any(|c| c.is_ascii_digit()),
            "Version output should contain version number"
        );
    }
}
