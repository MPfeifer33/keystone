use std::path::Path;
use serde::{Deserialize, Serialize};

use crate::KeystoneError;

const CONTRACT_FILE: &str = ".agent-contract.toml";

#[derive(Debug, Serialize, Deserialize)]
pub struct Contract {
    /// Project metadata
    #[serde(default)]
    pub project: ProjectMeta,
    /// Safe edit zones — agents can freely modify these
    #[serde(default)]
    pub safe_zones: Vec<ZoneRule>,
    /// Protected zones — require explicit approval
    #[serde(default)]
    pub protected: Vec<ZoneRule>,
    /// Validation rules — must pass before commit
    #[serde(default)]
    pub validation: Vec<ValidationRule>,
    /// Ownership assignments
    #[serde(default)]
    pub ownership: Vec<OwnershipRule>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ProjectMeta {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub required_files: Vec<String>,
    #[serde(default)]
    pub style: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ZoneRule {
    pub pattern: String,
    #[serde(default)]
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationRule {
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OwnershipRule {
    pub pattern: String,
    pub owner: String,
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            project: ProjectMeta {
                name: String::new(),
                required_files: vec!["PROJECT.md".into(), "README.md".into()],
                style: vec!["composition over inheritance".into()],
            },
            safe_zones: vec![
                ZoneRule { pattern: "src/**".into(), reason: "Source code".into() },
                ZoneRule { pattern: "tests/**".into(), reason: "Test code".into() },
                ZoneRule { pattern: "docs/**".into(), reason: "Documentation".into() },
            ],
            protected: vec![
                ZoneRule { pattern: "Cargo.toml".into(), reason: "Manifest — review dependency changes".into() },
                ZoneRule { pattern: ".github/**".into(), reason: "CI/CD — review pipeline changes".into() },
                ZoneRule { pattern: ".env*".into(), reason: "Environment config — may contain secrets".into() },
            ],
            validation: vec![
                ValidationRule { name: "build".into(), command: "cargo check".into(), required: true },
                ValidationRule { name: "test".into(), command: "cargo test".into(), required: true },
                ValidationRule { name: "rivet".into(), command: "rivet check".into(), required: false },
            ],
            ownership: vec![],
        }
    }
}

pub fn load(repo: &Path) -> Result<Option<Contract>, KeystoneError> {
    let path = repo.join(CONTRACT_FILE);
    if !path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&path)?;
    let contract: Contract = toml::from_str(&content)
        .map_err(|e| KeystoneError::Validation(format!("Invalid contract: {e}")))?;
    Ok(Some(contract))
}

pub fn save(repo: &Path, contract: &Contract) -> Result<(), KeystoneError> {
    let path = repo.join(CONTRACT_FILE);
    let content = toml::to_string_pretty(contract)
        .map_err(|e| KeystoneError::Validation(format!("Failed to serialize: {e}")))?;
    std::fs::write(&path, content)?;
    Ok(())
}

pub fn contract_exists(repo: &Path) -> bool {
    repo.join(CONTRACT_FILE).exists()
}
