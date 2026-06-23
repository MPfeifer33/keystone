use std::path::Path;
use regex::Regex;
use serde::Serialize;

use crate::contract::Contract;

#[derive(Debug, Serialize)]
pub struct LintResult {
    pub findings: Vec<LintFinding>,
    pub pass: bool,
}

#[derive(Debug, Serialize)]
pub struct LintFinding {
    pub rule: String,
    pub severity: LintSeverity,
    pub message: String,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LintSeverity {
    Error,
    Warning,
    Info,
}

pub fn lint(repo: &Path, contract: &Contract) -> LintResult {
    let mut findings = Vec::new();

    // Check required files
    for required in &contract.project.required_files {
        if !repo.join(required).exists() {
            findings.push(LintFinding {
                rule: "required_file".into(),
                severity: LintSeverity::Warning,
                message: format!("Required file missing: {required}"),
            });
        }
    }

    // Check for modified protected files in current diff
    if let Ok(dirty_files) = get_dirty_files(repo) {
        for file in &dirty_files {
            for protected in &contract.protected {
                if matches_pattern(file, &protected.pattern) {
                    findings.push(LintFinding {
                        rule: "protected_zone".into(),
                        severity: LintSeverity::Warning,
                        message: format!("Modified protected file: {file} — {}", protected.reason),
                    });
                }
            }
        }
    }

    // Check validation commands
    for validation in &contract.validation {
        let parts: Vec<&str> = validation.command.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        // Check if the tool exists
        let tool_exists = std::process::Command::new("which")
            .arg(parts[0])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !tool_exists && validation.required {
            findings.push(LintFinding {
                rule: "validation_tool".into(),
                severity: LintSeverity::Warning,
                message: format!("Required validation tool not found: {} (for `{}`)",
                    parts[0], validation.name),
            });
        }
    }

    let pass = !findings.iter().any(|f| matches!(f.severity, LintSeverity::Error));

    LintResult { findings, pass }
}

fn get_dirty_files(repo: &Path) -> Result<Vec<String>, std::io::Error> {
    let output = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(repo)
        .output()?;

    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.len() > 3 {
                Some(trimmed[3..].to_string())
            } else {
                None
            }
        })
        .collect())
}

fn matches_pattern(path: &str, pattern: &str) -> bool {
    if pattern.ends_with("/**") {
        let prefix = &pattern[..pattern.len() - 3];
        path.starts_with(prefix)
    } else if pattern.contains('*') {
        let regex_pattern = pattern
            .replace('.', r"\.")
            .replace('*', ".*");
        Regex::new(&format!("^{}$", regex_pattern))
            .map(|re| re.is_match(path))
            .unwrap_or(false)
    } else {
        path == pattern
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pattern_exact_match() {
        assert!(matches_pattern("Cargo.toml", "Cargo.toml"));
        assert!(!matches_pattern("Cargo.lock", "Cargo.toml"));
    }

    #[test]
    fn pattern_glob_match() {
        assert!(matches_pattern(".env.production", ".env*"));
        assert!(matches_pattern(".env", ".env*"));
    }

    #[test]
    fn pattern_dir_glob() {
        assert!(matches_pattern("src/main.rs", "src/**"));
        assert!(matches_pattern("src/deep/nested.rs", "src/**"));
        assert!(!matches_pattern("tests/foo.rs", "src/**"));
    }
}
